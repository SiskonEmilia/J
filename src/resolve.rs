use crate::error::JError;
use crate::merge::effective_children;
use crate::model::{Config, Node};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct JumpResult {
    pub abs_path: String,
    pub post_argv: Option<Vec<String>>,
}

pub fn resolve(
    positional: &[&str], // [root, sym1, sym2, ...]
    alias: Option<&str>,
    alias_args: &[String],
    c: &Config,
) -> Result<JumpResult, JError> {
    let (root_name, rest) = positional.split_first().ok_or_else(|| JError::Internal {
        msg: "resolve called with no positional args".into(),
    })?;

    let root = c.roots.get(*root_name).ok_or_else(|| JError::UnknownRoot {
        name: root_name.to_string(),
        available: c.roots.keys().cloned().collect(),
    })?;

    let mut cur_path = PathBuf::from(&root.path);
    let mut breadcrumb: Vec<String> = vec![root_name.to_string()];

    // 我们改用双路径：先沿字面 Node 走，一旦走到"来自模板的合成节点"就切换到
    // ResolvedChild 层面递归下钻。

    enum Cursor<'a> {
        Literal(&'a Node),
        Synth(crate::merge::ResolvedChild),
    }
    let mut cur = Cursor::Literal(root);

    for sym in rest {
        match cur {
            Cursor::Literal(n) => {
                let view = effective_children(n, c);
                let next = view.get(*sym).ok_or_else(|| JError::UnknownSymbol {
                    path: breadcrumb.clone(),
                    sym: sym.to_string(),
                    available: sources_listing(&view),
                })?;
                cur_path.push(&next.path);
                breadcrumb.push(sym.to_string());
                // 是否存在字面 Node 对应？若在 n.children 里则继续走 Literal
                cur = if let Some(child_node) = n.children.get(*sym) {
                    // 继续 Literal 可复用 effective_children 下一轮自然合并
                    Cursor::Literal(child_node)
                } else {
                    Cursor::Synth(next.clone())
                };
            }
            Cursor::Synth(rc) => {
                let next = rc
                    .children
                    .get(*sym)
                    .ok_or_else(|| JError::UnknownSymbol {
                        path: breadcrumb.clone(),
                        sym: sym.to_string(),
                        available: rc
                            .children
                            .iter()
                            .map(|(k, v)| (k.clone(), Some(v.source.clone())))
                            .collect(),
                    })?
                    .clone();
                cur_path.push(&next.path);
                breadcrumb.push(sym.to_string());
                cur = Cursor::Synth(next);
            }
        }
    }

    let post_argv = resolve_alias_argv(alias, alias_args, c)?;

    Ok(JumpResult {
        abs_path: cur_path.to_string_lossy().into_owned(),
        post_argv,
    })
}

pub fn resolve_current_alias(
    alias: &str,
    alias_args: &[String],
    c: &Config,
    current_dir: String,
) -> Result<JumpResult, JError> {
    Ok(JumpResult {
        abs_path: current_dir,
        post_argv: resolve_alias_argv(Some(alias), alias_args, c)?,
    })
}

fn resolve_alias_argv(
    alias: Option<&str>,
    alias_args: &[String],
    c: &Config,
) -> Result<Option<Vec<String>>, JError> {
    let Some(a) = alias else {
        return Ok(None);
    };

    let cmd_str = c.commands.get(a).ok_or_else(|| JError::UnknownAlias {
        name: a.to_string(),
        available: c.commands.keys().cloned().collect(),
    })?;
    let mut argv: Vec<String> = cmd_str.split_whitespace().map(String::from).collect();
    argv.extend(alias_args.iter().cloned());
    Ok(Some(argv))
}

fn sources_listing(
    view: &BTreeMap<String, crate::merge::ResolvedChild>,
) -> Vec<(String, Option<String>)> {
    view.iter()
        .map(|(k, v)| (k.clone(), Some(v.source.clone())))
        .collect()
}
