use crate::error::JError;
use jsonc_parser::cst::{CstObject, CstRootNode};
use jsonc_parser::cst::CstInputValue;
use jsonc_parser::ParseOptions;

pub struct CstDoc {
    root: CstRootNode,
}

impl std::fmt::Display for CstDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root)
    }
}

pub fn load_cst(src: &str) -> Result<CstDoc, JError> {
    let src = crate::config::strip_bom(src);
    let root = CstRootNode::parse(
        src,
        &ParseOptions {
            allow_comments: true,
            allow_trailing_commas: true,
            allow_loose_object_property_names: false,
        },
    )
    .map_err(|e| JError::ConfigError {
        path: "".into(),
        line: e.line_display(),
        col: e.column_display(),
        msg: e.kind().to_string(),
    })?;
    Ok(CstDoc { root })
}

impl CstDoc {
    fn top(&self) -> Result<CstObject, JError> {
        self.root
            .object_value()
            .ok_or_else(|| JError::ConfigInvalid {
                msg: "top-level must be an object".into(),
            })
    }

    /// Upsert a root entry: `"roots": { <name>: { "path": <path> } }`.
    pub fn upsert_root(&mut self, name: &str, path: &str) -> Result<(), JError> {
        let roots = self.ensure_key_as_object("roots")?;
        upsert_prop(
            &roots,
            name,
            CstInputValue::Object(vec![(
                "path".into(),
                CstInputValue::String(escape_for_cst(path)),
            )]),
        );
        Ok(())
    }

    /// Remove a root entry by name (no-op if absent).
    pub fn remove_root(&mut self, name: &str) -> Result<(), JError> {
        let roots = self.ensure_key_as_object("roots")?;
        remove_prop(&roots, name);
        Ok(())
    }

    /// Upsert a command alias: `"commands": { <name>: <cmd> }`.
    pub fn set_alias(&mut self, name: &str, cmd: &str) -> Result<(), JError> {
        let cmds = self.ensure_key_as_object("commands")?;
        upsert_prop(&cmds, name, CstInputValue::String(escape_for_cst(cmd)));
        Ok(())
    }

    /// Remove a command alias (no-op if absent).
    pub fn remove_alias(&mut self, name: &str) -> Result<(), JError> {
        let cmds = self.ensure_key_as_object("commands")?;
        remove_prop(&cmds, name);
        Ok(())
    }

    /// Upsert a node at `roots.<root>.children.<sym[0]>.children.<sym[1]>...`.
    /// `symbols` must be non-empty; use `upsert_root` for the root itself.
    pub fn upsert_node_path(
        &mut self,
        root: &str,
        symbols: &[&str],
        path: &str,
    ) -> Result<(), JError> {
        assert!(!symbols.is_empty(), "use upsert_root for root-level nodes");
        let top = self.top()?;
        let roots_obj = top.object_value_or_set("roots");
        let root_obj = roots_obj.object_value_or_set(root);
        let mut cur = root_obj;
        for sym in &symbols[..symbols.len() - 1] {
            let children = cur.object_value_or_set("children");
            cur = children.object_value_or_set(sym);
        }
        let children = cur.object_value_or_set("children");
        let last = symbols.last().unwrap();
        upsert_prop(
            &children,
            last,
            CstInputValue::Object(vec![(
                "path".into(),
                CstInputValue::String(escape_for_cst(path)),
            )]),
        );
        Ok(())
    }

    /// Remove a node. If `symbols` is empty, removes the root itself.
    pub fn remove_node(&mut self, root: &str, symbols: &[&str]) -> Result<(), JError> {
        if symbols.is_empty() {
            return self.remove_root(root);
        }
        let top = self.top()?;
        let Some(roots_obj) = top.object_value("roots") else {
            return Ok(()); // nothing to remove
        };
        let Some(root_obj) = roots_obj.object_value(root) else {
            return Ok(());
        };
        let mut cur = root_obj;
        for sym in &symbols[..symbols.len() - 1] {
            let Some(children) = cur.object_value("children") else {
                return Ok(());
            };
            let Some(next) = children.object_value(sym) else {
                return Ok(());
            };
            cur = next;
        }
        if let Some(children) = cur.object_value("children") {
            remove_prop(&children, symbols.last().unwrap());
        }
        Ok(())
    }

    /// Upsert a template entry. `children_json` is already-serialised JSONC.
    /// If the template exists and `force` is false, returns an error.
    pub fn upsert_template_from_subtree(
        &mut self,
        tpl_name: &str,
        children: &[(String, CstInputValue)],
        force: bool,
    ) -> Result<(), JError> {
        let tpls = self.ensure_key_as_object("templates")?;
        if tpls.get(tpl_name).is_some() && !force {
            return Err(JError::ConfigInvalid {
                msg: format!(
                    "template '{}' already exists (pass --force to overwrite)",
                    tpl_name
                ),
            });
        }
        upsert_prop(
            &tpls,
            tpl_name,
            CstInputValue::Object(vec![(
                "children".into(),
                CstInputValue::Object(children.to_vec()),
            )]),
        );
        Ok(())
    }

    /// Append `tpl_name` to the target node's `"templates": [...]` if absent.
    /// `symbols` may be empty to target the root node itself.
    pub fn apply_template_ref(
        &mut self,
        root: &str,
        symbols: &[&str],
        tpl_name: &str,
    ) -> Result<(), JError> {
        let top = self.top()?;
        let roots_obj = top.object_value_or_set("roots");
        let mut cur = roots_obj.object_value_or_set(root);
        for sym in symbols {
            let children = cur.object_value_or_set("children");
            cur = children.object_value_or_set(sym);
        }

        let templates = cur.array_value_or_set("templates");
        let exists = templates.elements().iter().any(|elem| {
            elem.as_string_lit()
                .and_then(|lit| lit.decoded_value().ok())
                .is_some_and(|decoded| decoded == tpl_name)
        });
        if !exists {
            templates.append(CstInputValue::String(tpl_name.to_string()));
        }
        Ok(())
    }

    /// Remove a template. If `force` is false and it is still referenced, returns an error.
    pub fn remove_template(
        &mut self,
        name: &str,
        force: bool,
        referenced_by: &[String],
    ) -> Result<(), JError> {
        if !force && !referenced_by.is_empty() {
            return Err(JError::ConfigInvalid {
                msg: format!(
                    "template '{}' is referenced by: {}",
                    name,
                    referenced_by.join(", ")
                ),
            });
        }
        let tpls = self.ensure_key_as_object("templates")?;
        remove_prop(&tpls, name);
        Ok(())
    }

    /// Remove all occurrences of `tpl_name` from `"templates": [...]` arrays
    /// within every node in the `"roots"` subtree. Call after `remove_template`
    /// when `force` is true so that dangling references are cleaned up.
    pub fn strip_template_refs(&mut self, tpl_name: &str) -> Result<(), JError> {
        let top = self.top()?;
        let Some(roots_obj) = top.object_value("roots") else {
            return Ok(());
        };
        for prop in roots_obj.properties() {
            if let Some(root_node) = prop.value().and_then(|v| v.as_object()) {
                strip_template_refs_from_node(&root_node, tpl_name);
            }
        }
        Ok(())
    }

    /// Get (or create as empty object) the top-level key `name`.
    fn ensure_key_as_object(&mut self, name: &str) -> Result<CstObject, JError> {
        let top = self.top()?;
        Ok(top.object_value_or_set(name))
    }
}

/// Upsert a property on `obj`: if the key exists, replace its value; otherwise append.
fn upsert_prop(obj: &CstObject, key: &str, value: CstInputValue) {
    if let Some(existing) = obj.get(key) {
        existing.set_value(value);
    } else {
        obj.append(key, value);
    }
}

/// Remove a property from `obj` (no-op if absent).
fn remove_prop(obj: &CstObject, key: &str) {
    if let Some(prop) = obj.get(key) {
        prop.remove();
    }
}

/// Pre-escape a string for `CstInputValue::String`.
///
/// `jsonc_parser`'s `CstStringLit::new_escaped` only escapes double-quotes,
/// not backslashes.  We must therefore hand the library the already-escaped
/// form so that Windows paths like `D:\d4` are stored as `D:\\d4` in the
/// JSON text.
pub fn escape_for_cst(s: &str) -> String {
    s.replace('\\', "\\\\")
}

/// Recursively strip all occurrences of `tpl_name` from `"templates": [...]`
/// arrays within a node object and its children.
fn strip_template_refs_from_node(obj: &CstObject, tpl_name: &str) {
    // Strip from the "templates" array on this node
    if let Some(tpls_array) = obj.array_value("templates") {
        for elem in tpls_array.elements() {
            if let Some(lit) = elem.as_string_lit() {
                if let Ok(decoded) = lit.decoded_value() {
                    if decoded == tpl_name {
                        lit.remove();
                    }
                }
            }
        }
    }
    // Recurse into children
    if let Some(children_obj) = obj.object_value("children") {
        for prop in children_obj.properties() {
            if let Some(child_node) = prop.value().and_then(|v| v.as_object()) {
                strip_template_refs_from_node(&child_node, tpl_name);
            }
        }
    }
}
