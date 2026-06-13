pub mod parse;
pub mod validate;
pub mod write;

pub use parse::load_from_str;
use crate::error::JError;
use crate::model::Config;
use std::path::Path;

/// Strip a leading UTF-8 BOM (U+FEFF) if present.
///
/// Windows editors such as Notepad save UTF-8 files with a BOM by default.
/// Left in place, the BOM makes the JSONC parser reject the file (the leading
/// byte sequence is not valid JSON whitespace) and would also corrupt CST
/// round-tripping. Stripping it on read lets such configs load, and rewrites
/// them back as plain (BOM-less) UTF-8.
pub fn strip_bom(src: &str) -> &str {
    src.strip_prefix('\u{feff}').unwrap_or(src)
}

pub fn load_from_str_validated(src: &str, path: &str) -> Result<Config, JError> {
    let c = parse::load_from_str(src, path)?;
    validate::validate(&c)?;
    Ok(c)
}

/// Read config file source, returning `"{}"` if the file does not yet exist.
/// Other IO errors are propagated.
pub fn read_or_empty(cfg_path: &Path) -> Result<String, JError> {
    match std::fs::read_to_string(cfg_path) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("{}".into()),
        Err(e) => Err(JError::ConfigError {
            path: cfg_path.display().to_string(),
            line: 0, col: 0,
            msg: format!("read: {}", e),
        }),
    }
}

/// Create parent directories of `cfg_path` if they don't exist.
pub fn ensure_parent_dir(cfg_path: &Path) -> Result<(), JError> {
    if let Some(parent) = cfg_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| JError::ConfigError {
            path: cfg_path.display().to_string(),
            line: 0, col: 0,
            msg: format!("mkdir: {}", e),
        })?;
    }
    Ok(())
}
