use crate::error::JError;
use std::path::Path;
use std::process::Command;

pub fn run(path: &Path) -> Result<String, JError> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| JError::InstallError {
                msg: format!("cannot create {}: {}", parent.display(), e),
            })?;
        }
        std::fs::write(path, DEFAULT).map_err(|e| JError::InstallError {
            msg: format!("cannot create {}: {}", path.display(), e),
        })?;
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".into());
    let status = Command::new(&editor)
        .arg(path)
        .status()
        .map_err(|e| JError::InstallError {
            msg: format!("cannot launch editor '{}': {}", editor, e),
        })?;
    if !status.success() {
        return Err(JError::InstallError {
            msg: format!("editor '{}' exited non-zero", editor),
        });
    }
    Ok(String::new())
}

const DEFAULT: &str = r#"{
  // commands: alias name -> command string (whitespace-split)
  "commands": {},
  // templates: reusable subtrees
  "templates": {},
  // roots: absolute-path starting points
  "roots": {}
}
"#;
