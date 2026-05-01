use crate::config::write::load_cst;
use crate::error::JError;
use std::path::Path;

pub fn run(cfg_path: &Path, args: &[String]) -> Result<String, JError> {
    if args.is_empty() {
        return Err(JError::ConfigInvalid { msg: "usage: j :rm <root> [<sym>...]".into() });
    }
    let src = std::fs::read_to_string(cfg_path).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0, msg: format!("read: {}", e),
    })?;
    let mut doc = load_cst(&src)?;
    let syms: Vec<&str> = args[1..].iter().map(String::as_str).collect();
    doc.remove_node(&args[0], &syms)?;
    std::fs::write(cfg_path, doc.to_string()).map_err(|e| JError::ConfigError {
        path: cfg_path.display().to_string(), line: 0, col: 0, msg: format!("write: {}", e),
    })?;
    Ok(String::new())
}
