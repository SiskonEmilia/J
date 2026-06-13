// Task 23
use crate::error::JError;
use crate::install::EnvWriter;
use std::path::{Path, PathBuf};

pub fn build_shim_bat(exe_abs: &str) -> String {
    format!(
r#"@echo off
setlocal enabledelayedexpansion
rem j.exe always writes its output as UTF-8. On a non-ASCII (e.g. CJK) system
rem the console code page defaults to OEM (936/GBK, etc.), so `call`/`type` would
rem decode the temp script and listing output as the wrong charset and mangle
rem non-ASCII paths. Capture the active code page, switch to UTF-8 (65001) for
rem the duration, and restore it afterwards.
rem Assumes chcp prints "<label>: <number>" with an ASCII colon, which holds on
rem localized Windows incl. zh-CN ("活动代码页: 936"). If a locale ever used a
rem non-ASCII colon, _J_CP would be empty and the restore below is a no-op (the
rem console simply stays on UTF-8) -- benign, never a failure.
for /f "tokens=2 delims=:" %%c in ('chcp') do set "_J_CP=%%c"
set "_J_CP=!_J_CP: =!"
chcp 65001 >nul
set "_J_TMP=%TEMP%\j_%RANDOM%%RANDOM%.bat"
"{exe}" --shell=cmd %* > "%_J_TMP%"
set _J_RC=%ERRORLEVEL%
rem In the jump branch below, endlocal makes the `cd` persist in the caller's
rem shell, so %_J_CP% is expanded at parse time, before endlocal discards it.
rem chcp restores the original code page after the call, which still needs UTF-8.
if %_J_RC% EQU 0 (
    findstr /b "cd /d" "%_J_TMP%" >nul 2>&1
    if !ERRORLEVEL! EQU 0 (
        endlocal & call "%_J_TMP%" & chcp %_J_CP% >nul & del "%_J_TMP%" >nul 2>&1
    ) else (
        type "%_J_TMP%"
        chcp !_J_CP! >nul
        del "%_J_TMP%" >nul 2>&1
        endlocal
    )
) else (
    chcp !_J_CP! >nul
    del "%_J_TMP%" >nul 2>&1
    endlocal
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
