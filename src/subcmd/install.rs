use crate::error::JError;
#[cfg(windows)]
use crate::install::WinRegEnvWriter;
use crate::install::{cmd as cmd_install, posix, powershell};
use std::path::Path;

pub fn install(_cfg_path: &Path, args: &[String]) -> Result<String, JError> {
    let shell = args
        .first()
        .map(String::as_str)
        .ok_or_else(|| JError::InstallError {
            msg: "usage: j :install <powershell|cmd|zsh|bash|sh>".into(),
        })?;
    let exe = std::env::current_exe().map_err(|e| JError::InstallError {
        msg: format!("current_exe: {}", e),
    })?;
    let exe_str = exe.display().to_string();
    match shell {
        "powershell" => {
            let profiles = powershell::all_profile_paths()?;
            let mut installed = Vec::new();
            for p in &profiles {
                powershell::install_into_file(p, &exe_str)?;
                installed.push(p.display().to_string());
            }
            Ok(format!(
                "installed PowerShell shim into:\n{}\n",
                installed
                    .iter()
                    .map(|s| format!("  {}", s))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        }
        "cmd" => {
            #[cfg(windows)]
            {
                cmd_install::install(&exe_str, &WinRegEnvWriter)?;
                Ok("installed cmd shim (j.bat + HKCU Path)\n".into())
            }
            #[cfg(not(windows))]
            Err(JError::InstallError {
                msg: "cmd install only supported on Windows".into(),
            })
        }
        "zsh" | "bash" | "sh" => {
            let profile = posix::profile_path(shell)?;
            posix::install_into_file(&profile, &exe_str, shell)?;
            Ok(format!(
                "installed {} shim into:\n  {}\n",
                shell,
                profile.display()
            ))
        }
        _ => Err(JError::InstallError {
            msg: format!("unknown shell '{}' (use powershell|cmd|zsh|bash|sh)", shell),
        }),
    }
}

pub fn uninstall(_cfg_path: &Path, args: &[String]) -> Result<String, JError> {
    let shell = args
        .first()
        .map(String::as_str)
        .ok_or_else(|| JError::InstallError {
            msg: "usage: j :uninstall <powershell|cmd|zsh|bash|sh>".into(),
        })?;
    match shell {
        "powershell" => {
            let profiles = powershell::all_profile_paths()?;
            let mut removed = Vec::new();
            for p in &profiles {
                powershell::uninstall_from_file(p)?;
                removed.push(p.display().to_string());
            }
            Ok(format!(
                "removed PowerShell shim from:\n{}\n",
                removed
                    .iter()
                    .map(|s| format!("  {}", s))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        }
        "cmd" => {
            #[cfg(windows)]
            {
                cmd_install::uninstall(&WinRegEnvWriter)?;
                Ok("removed cmd shim\n".into())
            }
            #[cfg(not(windows))]
            Err(JError::InstallError {
                msg: "cmd uninstall only supported on Windows".into(),
            })
        }
        "zsh" | "bash" | "sh" => {
            let profile = posix::profile_path(shell)?;
            posix::uninstall_from_file(&profile)?;
            Ok(format!(
                "removed {} shim from:\n  {}\n",
                shell,
                profile.display()
            ))
        }
        _ => Err(JError::InstallError {
            msg: format!("unknown shell '{}'", shell),
        }),
    }
}

pub fn init(args: &[String]) -> Result<String, JError> {
    let shell = args
        .first()
        .map(String::as_str)
        .ok_or_else(|| JError::InstallError {
            msg: "usage: j :init <powershell|cmd|zsh|bash|sh>".into(),
        })?;
    let exe = std::env::current_exe().map_err(|e| JError::InstallError {
        msg: format!("current_exe: {}", e),
    })?;
    let exe_str = exe.display().to_string();
    let s = match shell {
        "powershell" => powershell::build_shim_script(&exe_str),
        "cmd" => cmd_install::build_shim_bat(&exe_str),
        "zsh" | "bash" | "sh" => posix::build_shim_script(&exe_str, shell),
        _ => {
            return Err(JError::InstallError {
                msg: format!("unknown shell '{}'", shell),
            })
        }
    };
    Ok(s)
}
