use crate::config::{load_from_str_validated, write::{escape_for_cst, load_cst}};
use crate::error::JError;
use crate::merge::{effective_children, ResolvedChild};
use crate::model::Node;
use jsonc_parser::cst::CstInputValue;
use std::collections::BTreeMap;
use std::path::Path;

pub fn dump(cfg_path: &Path, args: &[String]) -> Result<String, JError> {
    let (force, rest) = take_force(args);
    if rest.len() < 2 {
        return Err(JError::ConfigInvalid {
            msg: "usage: j :tpl-dump [--force] <root> <sym>... <tpl-name>".into(),
        });
    }
    let tpl_name = rest.last().unwrap();
    let root_name = &rest[0];
    let syms: Vec<&str> = rest[1..rest.len() - 1].iter().map(String::as_str).collect();

    let src = std::fs::read_to_string(cfg_path).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0, msg: format!("read: {}", e),
    })?;
    let cfg = load_from_str_validated(&src, &cfg_path.display().to_string())?;

    // compute merged children at the target node
    let root = cfg.roots.get(root_name).ok_or_else(|| JError::UnknownRoot {
        name: root_name.clone(), available: cfg.roots.keys().cloned().collect(),
    })?;
    let children = subtree_children_at(root, &cfg, &syms)?;
    let children_cst = resolved_to_cst_children(&children);

    let mut doc = load_cst(&src)?;
    doc.upsert_template_from_subtree(tpl_name, &children_cst, force)?;
    std::fs::write(cfg_path, doc.to_string()).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0, msg: format!("write: {}", e),
    })?;
    Ok(String::new())
}

pub fn remove(cfg_path: &Path, args: &[String]) -> Result<String, JError> {
    let (force, rest) = take_force(args);
    if rest.len() != 1 {
        return Err(JError::ConfigInvalid { msg: "usage: j :tpl-rm [--force] <tpl-name>".into() });
    }
    let name = &rest[0];
    let src = std::fs::read_to_string(cfg_path).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0, msg: format!("read: {}", e),
    })?;
    let cfg = load_from_str_validated(&src, &cfg_path.display().to_string())?;
    let refs = collect_refs(name, &cfg);

    let mut doc = load_cst(&src)?;
    doc.remove_template(name, force, &refs)?;
    if force {
        doc.strip_template_refs(name)?;
    }
    std::fs::write(cfg_path, doc.to_string()).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0, msg: format!("write: {}", e),
    })?;
    Ok(String::new())
}

pub fn apply(cfg_path: &Path, args: &[String]) -> Result<String, JError> {
    if args.len() < 2 {
        return Err(JError::ConfigInvalid {
            msg: "usage: j :tpl-apply <root> [<sym>...] <tpl-name>".into(),
        });
    }
    let tpl_name = args.last().unwrap();
    let root_name = &args[0];
    let syms: Vec<&str> = args[1..args.len() - 1].iter().map(String::as_str).collect();

    let src = std::fs::read_to_string(cfg_path).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0, msg: format!("read: {}", e),
    })?;
    let cfg = load_from_str_validated(&src, &cfg_path.display().to_string())?;

    if !cfg.templates.contains_key(tpl_name) {
        return Err(JError::ConfigInvalid {
            msg: format!("unknown template '{}'", tpl_name),
        });
    }
    let root = cfg.roots.get(root_name).ok_or_else(|| JError::UnknownRoot {
        name: root_name.clone(), available: cfg.roots.keys().cloned().collect(),
    })?;
    literal_node_at(root, &syms)?;

    let mut doc = load_cst(&src)?;
    doc.apply_template_ref(root_name, &syms, tpl_name)?;
    std::fs::write(cfg_path, doc.to_string()).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0, msg: format!("write: {}", e),
    })?;
    Ok(String::new())
}

fn take_force(args: &[String]) -> (bool, Vec<String>) {
    let mut force = false;
    let mut out: Vec<String> = Vec::with_capacity(args.len());
    for a in args {
        if a == "--force" { force = true; } else { out.push(a.clone()); }
    }
    (force, out)
}

fn literal_node_at<'a>(root: &'a Node, syms: &[&str]) -> Result<&'a Node, JError> {
    let mut cur = root;
    for (i, sym) in syms.iter().enumerate() {
        cur = cur.children.get(*sym).ok_or_else(|| JError::UnknownSymbol {
            path: syms[..i].iter().map(|s| s.to_string()).collect(),
            sym: sym.to_string(),
            available: cur.children.keys().map(|k| (k.clone(), None)).collect(),
        })?;
    }
    Ok(cur)
}

fn subtree_children_at(root: &Node, cfg: &crate::model::Config, syms: &[&str])
    -> Result<BTreeMap<String, ResolvedChild>, JError>
{
    if syms.is_empty() {
        return Ok(effective_children(root, cfg));
    }
    // drill down
    let mut view = effective_children(root, cfg);
    for (i, s) in syms.iter().enumerate() {
        let next = view.get(*s).ok_or_else(|| JError::UnknownSymbol {
            path: syms[..i].iter().map(|s| s.to_string()).collect(),
            sym: s.to_string(),
            available: view.iter().map(|(k, v)| (k.clone(), Some(v.source.clone()))).collect(),
        })?.clone();
        view = next.children.clone();
    }
    Ok(view)
}

/// Build a `Vec<(String, CstInputValue)>` from a resolved children map.
/// This is the format accepted by `CstDoc::upsert_template_from_subtree`.
fn resolved_to_cst_children(view: &BTreeMap<String, ResolvedChild>) -> Vec<(String, CstInputValue)> {
    view.iter().map(|(k, v)| (k.clone(), resolved_child_to_cst(v))).collect()
}

fn resolved_child_to_cst(v: &ResolvedChild) -> CstInputValue {
    let mut props: Vec<(String, CstInputValue)> = vec![
        ("path".into(), CstInputValue::String(escape_for_cst(&v.path))),
    ];
    if !v.children.is_empty() {
        props.push(("children".into(), CstInputValue::Object(resolved_to_cst_children(&v.children))));
    }
    CstInputValue::Object(props)
}

fn collect_refs(tpl_name: &str, cfg: &crate::model::Config) -> Vec<String> {
    let mut out = Vec::new();
    for (rname, root) in &cfg.roots {
        walk_for_ref(rname, root, tpl_name, &mut out);
    }
    out
}

fn walk_for_ref(crumb: &str, n: &Node, tpl: &str, out: &mut Vec<String>) {
    if n.templates.iter().any(|t| t == tpl) {
        out.push(crumb.to_string());
    }
    for (k, c) in &n.children {
        walk_for_ref(&format!("{}.{}", crumb, k), c, tpl, out);
    }
}
