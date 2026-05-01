mod common;
use common::exe;
use std::process::Command;
use tempfile::tempdir;
use std::fs;

fn seed(dir: &std::path::Path) -> std::path::PathBuf {
    let cfg = dir.join("config.jsonc");
    fs::write(&cfg, r#"{
  // preserved
  "commands": {},
  "roots": {
    "d3": { "path": "C:\\projects\\d3" }
  }
}"#).unwrap();
    cfg
}

#[test]
fn add_new_root() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());

    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":add", "d4", "D:\\projects\\d4"]);
    let o = c.output().unwrap();
    assert_eq!(o.status.code().unwrap(), 0, "stderr={:?}", String::from_utf8_lossy(&o.stderr));

    let s = fs::read_to_string(&cfg).unwrap();
    assert!(s.contains("\"d4\""));
    assert!(s.contains("D:\\\\projects\\\\d4"));
    assert!(s.contains("// preserved"));
}

#[test]
fn add_current_directory_as_root() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let target = dir.path().join("workspace");
    fs::create_dir_all(&target).unwrap();

    let mut c = Command::new(exe());
    c.current_dir(&target)
        .env("J_CONFIG", &cfg)
        .args([":add", "work", "."]);
    let o = c.output().unwrap();
    assert_eq!(o.status.code().unwrap(), 0, "stderr={:?}", String::from_utf8_lossy(&o.stderr));

    let s = fs::read_to_string(&cfg).unwrap();
    let expected = target.to_string_lossy().replace('\\', "\\\\");
    assert!(s.contains("\"work\""));
    assert!(s.contains(&expected), "expected path {expected} in config: {s}");
}

#[test]
fn add_creates_config_when_missing() {
    let dir = tempdir().unwrap();
    // Point to a file (and subdirectory) that do not yet exist
    let cfg = dir.path().join("subdir").join("config.jsonc");

    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":add", "d3", "C:\\projects\\d3"]);
    let o = c.output().unwrap();
    assert_eq!(o.status.code().unwrap(), 0, "stderr={:?}", String::from_utf8_lossy(&o.stderr));

    let s = fs::read_to_string(&cfg).unwrap();
    assert!(s.contains("\"d3\""), "root key missing: {s}");
    assert!(s.contains("C:\\\\projects\\\\d3"), "path missing: {s}");
}

#[test]
fn add_child_under_root() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());

    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":add", "d3", "notes", "docs\\notes"]);
    let o = c.output().unwrap();
    assert_eq!(o.status.code().unwrap(), 0, "stderr={:?}", String::from_utf8_lossy(&o.stderr));

    let s = fs::read_to_string(&cfg).unwrap();
    assert!(s.contains("\"notes\""));
    assert!(s.contains("docs\\\\notes"));
}
