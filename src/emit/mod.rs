pub mod cmd;
pub mod posix;
pub mod powershell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    PowerShell,
    Cmd,
    Posix,
}

pub fn emit(shell: Shell, abs_path: &str, post_argv: Option<&[String]>) -> String {
    match shell {
        Shell::PowerShell => powershell::emit(abs_path, post_argv),
        Shell::Cmd => cmd::emit(abs_path, post_argv),
        Shell::Posix => posix::emit(abs_path, post_argv),
    }
}
