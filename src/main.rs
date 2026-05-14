use std::io::Write;
use std::process::ExitCode;

use j::cli::{parse, Invocation, Shell};
use j::config::load_from_str_validated;
use j::emit;
use j::error::{exit_code, JError};
use j::resolve::{resolve, resolve_current_alias};
use j::subcmd;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let argv_ref: Vec<&str> = argv.iter().map(String::as_str).collect();

    match run(&argv_ref) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "{}", e);
            ExitCode::from(exit_code(&e) as u8)
        }
    }
}

fn run(argv: &[&str]) -> Result<(), JError> {
    match parse(argv)? {
        Invocation::Jump {
            positional,
            alias,
            alias_args,
            shell,
        } => {
            let shell = shell.ok_or_else(|| JError::Internal {
                msg: "jump form requires --shell=<powershell|cmd|zsh|bash|sh> (normally injected by shim)".into(),
            })?;
            let cfg_path = config_path();
            let src = std::fs::read_to_string(&cfg_path).map_err(|e| JError::ConfigError {
                path: cfg_path.display().to_string(),
                line: 0,
                col: 0,
                msg: format!("cannot read: {}", e),
            })?;
            let cfg = load_from_str_validated(&src, &cfg_path.display().to_string())?;

            let pos_ref: Vec<&str> = positional.iter().map(String::as_str).collect();
            let r = if positional.is_empty() {
                let alias = alias.as_deref().ok_or_else(|| JError::Internal {
                    msg: "jump form requires a root or alias".into(),
                })?;
                let cwd = std::env::current_dir().map_err(|e| JError::Internal {
                    msg: format!("cannot read current directory: {}", e),
                })?;
                resolve_current_alias(alias, &alias_args, &cfg, cwd.to_string_lossy().into_owned())?
            } else {
                resolve(&pos_ref, alias.as_deref(), &alias_args, &cfg)?
            };
            let shell_enum = match shell {
                Shell::PowerShell => emit::Shell::PowerShell,
                Shell::Cmd => emit::Shell::Cmd,
                Shell::Posix => emit::Shell::Posix,
            };
            let script = emit::emit(shell_enum, &r.abs_path, r.post_argv.as_deref());
            print!("{}", script);
            Ok(())
        }
        Invocation::Subcmd { name, args } => {
            let out = subcmd::dispatch(&name, &args, &config_path())?;
            print!("{}", out);
            Ok(())
        }
    }
}

pub fn config_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("J_CONFIG") {
        return std::path::PathBuf::from(p);
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    std::path::PathBuf::from(home)
        .join(".config")
        .join("j")
        .join("config.jsonc")
}
