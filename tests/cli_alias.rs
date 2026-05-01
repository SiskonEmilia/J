mod common;
use common::exe;
use std::process::Command;
use tempfile::tempdir;
use std::fs;

fn seed(dir: &std::path::Path) -> std::path::PathBuf {
    let cfg = dir.join("config.jsonc");
    fs::write(&cfg, r#"{"commands":{"c":"code"},"roots":{"d":{"path":"C:\\d"}}}"#).unwrap();
    cfg
}

#[test]
fn alias_creates_config_when_missing() {
    let dir = tempdir().unwrap();
    let cfg = dir.path().join("subdir").join("config.jsonc");

    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":alias", "c", "code"]);
    let o = c.output().unwrap();
    assert_eq!(o.status.code().unwrap(), 0, "stderr={:?}", String::from_utf8_lossy(&o.stderr));

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
