pub mod powershell;
pub mod cmd;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell { PowerShell, Cmd }

pub fn emit(shell: Shell, abs_path: &str, post_argv: Option<&[String]>) -> String {
    match shell {
        Shell::PowerShell => powershell::emit(abs_path, post_argv),
        Shell::Cmd        => cmd::emit(abs_path, post_argv),
    }
}
