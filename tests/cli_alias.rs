mod common;
use common::exe;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn seed(dir: &std::path::Path) -> std::path::PathBuf {
    let cfg = dir.join("config.jsonc");
    fs::write(
        &cfg,
        r#"{"commands":{"c":"code"},"roots":{"d":{"path":"C:\\d"}}}"#,
    )
    .unwrap();
    cfg
}

#[test]
fn alias_creates_config_when_missing() {
    let dir = tempdir().unwrap();
    let cfg = dir.path().join("subdir").join("config.jsonc");

    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":alias", "c", "code"]);
    let o = c.output().unwrap();
    assert_eq!(
        o.status.code().unwrap(),
        0,
        "stderr={:?}",
        String::from_utf8_lossy(&o.stderr)
    );

    let s = fs::read_to_string(&cfg).unwrap();
    assert!(s.contains("\"c\""), "alias key missing: {s}");
    assert!(s.contains("code"), "alias value missing: {s}");
}

#[test]
fn set_new_alias() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":alias", "g", "git status"]);
    assert_eq!(c.status().unwrap().code().unwrap(), 0);
    let s = fs::read_to_string(&cfg).unwrap();
    assert!(s.contains("\"g\""));
    assert!(s.contains("git status"));
}

#[test]
fn overwrite_alias() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":alias", "c", "codium"]);
    assert_eq!(c.status().unwrap().code().unwrap(), 0);
    let s = fs::read_to_string(&cfg).unwrap();
    assert!(s.contains("codium"));
    assert!(!s.contains("\"code\""));
}

#[test]
fn rm_alias() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":alias", "--rm", "c"]);
    assert_eq!(c.status().unwrap().code().unwrap(), 0);
    let s = fs::read_to_string(&cfg).unwrap();
    assert!(!s.contains("\"c\":"));
}

#[test]
fn rejects_alias_name_with_leading_dash() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let before = fs::read_to_string(&cfg).unwrap();

    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":alias", "-cx", "codex"]);
    let o = c.output().unwrap();

    assert_eq!(o.status.code().unwrap(), 3);
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert!(
        stderr.contains("alias name '-cx' must start with [A-Za-z0-9_]"),
        "stderr={stderr:?}"
    );
    assert_eq!(fs::read_to_string(&cfg).unwrap(), before);
}
