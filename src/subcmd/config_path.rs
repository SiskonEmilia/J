use crate::error::JError;
use std::path::Path;

pub fn run(path: &Path) -> Result<String, JError> {
    Ok(format!("{}\n", path.display()))
}
