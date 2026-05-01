use crate::config::{ensure_parent_dir, read_or_empty, write::load_cst};
use crate::error::JError;
use std::path::Path;

pub fn run(cfg_path: &Path, args: &[String]) -> Result<String, JError> {
    let src = read_or_empty(cfg_path)?;
    let mut doc = load_cst(&src)?;

    if args.len() == 2 && args[0] == "--rm" {
        doc.remove_alias(&args[1])?;
    } else if args.len() >= 2 {
        let name = &args[0];
        let cmd  = args[1..].join(" ");
        doc.set_alias(name, &cmd)?;
    } else {
        return Err(JError::ConfigInvalid {
            msg: "usage: j :alias <name> <command>  |  j :alias --rm <name>".into(),
        });
    }

    ensure_parent_dir(cfg_path)?;
    std::fs::write(cfg_path, doc.to_string()).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0, msg: format!("write: {}", e),
    })?;
    Ok(String::new())
}
