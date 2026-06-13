use j::install::cmd::{build_shim_bat, install_into_dir, uninstall_from_dir};
use j::install::{EnvWriter, MockEnvWriter};
use tempfile::tempdir;

#[test]
fn build_shim_bat_has_exe() {
    let s = build_shim_bat("C:\\tools\\j\\j.exe");
    assert!(s.contains("\"C:\\tools\\j\\j.exe\""));
    assert!(s.contains("call \"%_J_TMP%\""));
    assert!(s.contains("--shell=cmd"));
    // Smart dispatch: execute only when output starts with a cd command;
    // otherwise display it with `type` so subcommand text is not run as batch.
    assert!(s.contains("findstr /b \"cd /d\""), "shim must check for cd /d prefix");
    assert!(s.contains("type \"%_J_TMP%\""), "shim must display non-script output");
    // UTF-8: switch the console code page so CJK paths in the temp script and
    // `type` output decode correctly, then restore the original code page.
    assert!(s.contains("chcp 65001"), "shim must switch to UTF-8 code page");
    assert!(s.contains("chcp %_J_CP%"), "shim must restore the original code page");
}

#[test]
fn install_writes_bat_and_updates_path() {
    let dir = tempdir().unwrap();
    let bin = dir.path().join("bin");
    let mock = MockEnvWriter::new("C:\\Windows");
    install_into_dir(&bin, "C:\\tools\\j\\j.exe", &mock).unwrap();
    assert!(bin.join("j.bat").exists());
    assert!(mock.read_path().unwrap().contains(&bin.display().to_string()));

    // idempotent
    install_into_dir(&bin, "C:\\tools\\j\\j.exe", &mock).unwrap();
    assert_eq!(mock.read_path().unwrap().matches(&bin.display().to_string()).count(), 1);
}

#[test]
fn uninstall_removes_bat_and_path() {
    let dir = tempdir().unwrap();
    let bin = dir.path().join("bin");
    let mock = MockEnvWriter::new("C:\\Windows");
    install_into_dir(&bin, "C:\\tools\\j\\j.exe", &mock).unwrap();
    uninstall_from_dir(&bin, &mock).unwrap();
    assert!(!bin.join("j.bat").exists());
    assert!(!mock.read_path().unwrap().contains(&bin.display().to_string()));
}
