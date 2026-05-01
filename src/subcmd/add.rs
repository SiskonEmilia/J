use crate::config::{ensure_parent_dir, read_or_empty, write::load_cst};
use crate::error::JError;
use std::path::Path;

pub fn run(cfg_path: &Path, args: &[String]) -> Result<String, JError> {
    if args.len() < 2 {
        return Err(JError::ConfigInvalid {
            msg: "usage: j :add <root> [<sym>...] <path>".into(),
        });
    }
    let path_arg = args.last().unwrap();
    let root_name = &args[0];
    let middle = &args[1..args.len() - 1]; // sym...

    let src = read_or_empty(cfg_path)?;
    let mut doc = load_cst(&src)?;

    if middle.is_empty() {
        // only root + path -> add/overwrite root
        let root_path = materialize_root_path(path_arg)?;
        doc.upsert_root(root_name, &root_path)?;
    } else {
        let syms: Vec<&str> = middle.iter().map(String::as_str).collect();
        doc.upsert_node_path(root_name, &syms, path_arg)?;
    }

    ensure_parent_dir(cfg_path)?;
    std::fs::write(cfg_path, doc.to_string()).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0,
        msg: format!("write: {}", e),
    })?;
    Ok(String::new())
}

fn materialize_root_path(path_arg: &str) -> Result<String, JError> {
    if path_arg != "." {
        return Ok(path_arg.to_string());
    }

    let cwd = std::env::current_dir().map_err(|e| JError::ConfigInvalid {
        msg: format!("cannot resolve current directory for '.': {}", e),
    })?;
    Ok(cwd.to_string_lossy().into_owned())
}
