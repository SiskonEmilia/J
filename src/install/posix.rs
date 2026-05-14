use crate::error::JError;
use crate::install::region;
use std::path::{Path, PathBuf};

pub fn build_shim_script(exe_abs: &str, shell: &str) -> String {
    let exe_q = quote(exe_abs);
    let shell_q = quote(shell);
    format!(
        r#"j() {{
    if [ "$#" -eq 0 ]; then
        "{exe}" :help
        return $?
    fi

    _j_out=$("{exe}" --shell={shell} "$@")
    _j_rc=$?
    if [ $_j_rc -ne 0 ]; then
        return $_j_rc
    fi
    if [ -z "$_j_out" ]; then
        return 0
    fi

    case "$_j_out" in
        cd\ --*)
            eval "$_j_out"
            ;;
        *)
            printf '%s\n' "$_j_out"
            ;;
    esac
}}
"#,
        exe = exe_q,
        shell = shell_q
    )
}

pub fn install_into_file(profile: &Path, exe_abs: &str, shell: &str) -> Result<(), JError> {
    let existing = std::fs::read_to_string(profile).unwrap_or_default();
    if let Some(parent) = profile.parent() {
        std::fs::create_dir_all(parent).map_err(|e| JError::InstallError {
            msg: format!("mkdir {}: {}", parent.display(), e),
        })?;
    }
    let body = build_shim_script(exe_abs, shell);
    let updated = region::upsert(&existing, &body);
    std::fs::write(profile, updated).map_err(|e| JError::InstallError {
        msg: format!("write {}: {}", profile.display(), e),
    })
}

pub fn uninstall_from_file(profile: &Path) -> Result<(), JError> {
    let existing = match std::fs::read_to_string(profile) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };
    let updated = region::remove(&existing)?;
    std::fs::write(profile, updated).map_err(|e| JError::InstallError {
        msg: format!("write {}: {}", profile.display(), e),
    })
}

pub fn profile_path(shell: &str) -> Result<PathBuf, JError> {
    let home = std::env::var("HOME").map_err(|_| JError::InstallError {
        msg: "HOME not set".into(),
    })?;
    let filename = match shell {
        "zsh" => ".zshrc",
        "bash" => ".bashrc",
        "sh" | "posix" => ".profile",
        _ => {
            return Err(JError::InstallError {
                msg: format!("unknown POSIX shell '{}'", shell),
            })
        }
    };
    Ok(PathBuf::from(home).join(filename))
}

fn quote(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
