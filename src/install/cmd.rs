// Task 23
use crate::error::JError;
use crate::install::EnvWriter;
use std::path::{Path, PathBuf};

pub fn build_shim_bat(exe_abs: &str) -> String {
    format!(
r#"@echo off
setlocal enabledelayedexpansion
set "_J_TMP=%TEMP%\j_%RANDOM%%RANDOM%.bat"
"{exe}" --shell=cmd %* > "%_J_TMP%"
set _J_RC=%ERRORLEVEL%
if %_J_RC% EQU 0 (
    findstr /b "cd /d" "%_J_TMP%" >nul 2>&1
    if !ERRORLEVEL! EQU 0 (
        endlocal & call "%_J_TMP%" & del "%_J_TMP%" >nul 2>&1
    ) else (
        type "%_J_TMP%"
        del "%_J_TMP%" >nul 2>&1
    )
) else (
    del "%_J_TMP%" >nul 2>&1
    exit /b %_J_RC%
)
"#, exe = exe_abs)
}

pub fn install_into_dir(bin_dir: &Path, exe_abs: &str, env: &dyn EnvWriter) -> Result<(), JError> {
    std::fs::create_dir_all(bin_dir).map_err(|e| JError::InstallError {
        msg: format!("mkdir {}: {}", bin_dir.display(), e),
    })?;
    let bat = bin_dir.join("j.bat");
    std::fs::write(&bat, build_shim_bat(exe_abs)).map_err(|e| JError::InstallError {
        msg: format!("write {}: {}", bat.display(), e),
    })?;
    env.ensure_path_contains(&bin_dir.display().to_string())?;
    Ok(())
}

pub fn uninstall_from_dir(bin_dir: &Path, env: &dyn EnvWriter) -> Result<(), JError> {
    let bat = bin_dir.join("j.bat");
    if bat.exists() {
        std::fs::remove_file(&bat).map_err(|e| JError::InstallError {
            msg: format!("remove {}: {}", bat.display(), e),
        })?;
    }
    env.ensure_path_not_contains(&bin_dir.display().to_string())?;
    Ok(())
}

pub fn default_bin_dir() -> Result<PathBuf, JError> {
    let home = std::env::var("USERPROFILE").map_err(|_| JError::InstallError {
        msg: "USERPROFILE not set".into(),
    })?;
    Ok(PathBuf::from(home).join(".config").join("j").join("bin"))
}

/// Production version: uses default bin dir + real env writer.
pub fn install(exe_abs: &str, env: &dyn EnvWriter) -> Result<(), JError> {
    let bin = default_bin_dir()?;
    install_into_dir(&bin, exe_abs, env)
}

pub fn uninstall(env: &dyn EnvWriter) -> Result<(), JError> {
    let bin = default_bin_dir()?;
    uninstall_from_dir(&bin, env)
}
