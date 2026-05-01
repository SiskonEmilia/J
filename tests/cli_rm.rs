mod common;
use common::exe;
use std::process::Command;
use tempfile::tempdir;
use std::fs;

fn seed(dir: &std::path::Path) -> std::path::PathBuf {
    let cfg = dir.join("config.jsonc");
    fs::write(&cfg, r#"{
  "commands": { "c": "code" },
  "roots": {
    "d3": { "path": "C:\\d3", "children": { "notes": { "path": "n" } } },
    "d4": { "path": "D:\\d4" }
  }
}"#).unwrap();
    cfg
}

#[test]
fn rm_root() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":rm", "d4"]);
    assert_eq!(c.status().unwrap().code().unwrap(), 0);
    let s = fs::read_to_string(&cfg).unwrap();
    assert!(!s.contains("\"d4\""));
    assert!(s.contains("\"d3\""));
}

#[test]
fn rm_child() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":rm", "d3", "notes"]);
    assert_eq!(c.status().unwrap().code().unwrap(), 0);
    let s = fs::read_to_string(&cfg).unwrap();
    assert!(!s.contains("\"notes\""));
    assert!(s.contains("\"d3\""));
}
