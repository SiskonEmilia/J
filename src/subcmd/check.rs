use crate::error::JError;
use crate::merge::{effective_children, ResolvedChild};
use crate::model::{Config, Node};
use std::path::{Path, PathBuf};

pub fn run(cfg: &Config) -> Result<String, JError> {
    let mut ok: usize = 0;
    let mut missing: Vec<String> = Vec::new();

    for (name, root) in &cfg.roots {
        walk(&PathBuf::from(&root.path), root, cfg, name, &mut ok, &mut missing);
    }

    let mut out = String::new();
    out.push_str(&format!(
        "checked {} paths; {} ok, {} missing\n",
        ok + missing.len(),
        ok,
        missing.len()
    ));
    if missing.is_empty() {
        Ok(out)
    } else {
        for m in &missing {
            out.push_str(&format!("  missing: {}\n", m));
        }
        Err(JError::ConfigInvalid { msg: out.trim_end().to_string() })
    }
}

fn walk(
    abs: &Path,
    n: &Node,
    cfg: &Config,
    crumb: &str,
    ok: &mut usize,
    missing: &mut Vec<String>,
) {
    if abs.is_dir() {
        *ok += 1;
    } else {
        missing.push(format!("{} ({})", crumb, abs.display()));
    }
    let view = effective_children(n, cfg);
    for (k, v) in &view {
        let next = abs.join(&v.path);
        let sub_crumb = format!("{}.{}", crumb, k);
        if next.is_dir() {
            *ok += 1;
        } else {
            missing.push(format!("{} ({})", sub_crumb, next.display()));
        }
        walk_resolved(&next, v, &sub_crumb, ok, missing);
    }
}

fn walk_resolved(
    abs: &Path,
    rc: &ResolvedChild,
    crumb: &str,
    ok: &mut usize,
    missing: &mut Vec<String>,
) {
    for (k, v) in &rc.children {
        let next = abs.join(&v.path);
        let sub_crumb = format!("{}.{}", crumb, k);
        if next.is_dir() {
            *ok += 1;
        } else {
            missing.push(format!("{} ({})", sub_crumb, next.display()));
        }
        walk_resolved(&next, v, &sub_crumb, ok, missing);
    }
}
