use j::install::posix::{build_shim_script, install_into_file, uninstall_from_file};
use tempfile::tempdir;

#[test]
fn posix_shim_invokes_zsh_shell_mode() {
    let s = build_shim_script("/usr/local/bin/j", "zsh");
    assert!(s.contains("--shell=zsh"));
    assert!(s.contains("eval \"$_j_out\""));
}

#[test]
fn install_idempotent() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join(".zshrc");

    install_into_file(&profile, "/usr/local/bin/j", "zsh").unwrap();
    install_into_file(&profile, "/usr/local/bin/j", "zsh").unwrap();

    let s = std::fs::read_to_string(&profile).unwrap();
    assert_eq!(s.matches("# region j-shim").count(), 1);
    assert_eq!(s.matches("# endregion j-shim").count(), 1);
}

#[test]
fn uninstall_removes_region() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join(".zshrc");

    install_into_file(&profile, "/usr/local/bin/j", "zsh").unwrap();
    uninstall_from_file(&profile).unwrap();

    let s = std::fs::read_to_string(&profile).unwrap();
    assert!(!s.contains("BEGIN J"));
    assert!(!s.contains("j()"));
}
