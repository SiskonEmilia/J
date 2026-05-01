mod common;
use common::exe;
use std::process::Command;
use tempfile::tempdir;
use std::fs;

const SEED: &str = r#"{
  "templates": {
    "exist": { "children": { "x": { "path": "X" } } }
  },
  "roots": {
    "d3": {
      "path": "C:\\d3",
      "templates": ["exist"],
      "children": {
        "notes": { "path": "notes" }
      }
    }
  }
}"#;

fn seed(dir: &std::path::Path) -> std::path::PathBuf {
    let cfg = dir.join("config.jsonc");
    fs::write(&cfg, SEED).unwrap();
    cfg
}

#[test]
fn tpl_dump_root_children() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([":tpl-dump", "d3", "newTpl"]);
    assert_eq!(c.status().unwrap().code().unwrap(), 0);
    let s = fs::read_to_string(&cfg).unwrap();
    assert!(s.contains("\"newTpl\""));
    // merged children should include notes (from self) and x (from exist)
    assert!(s.contains("\"notes\""));
    assert!(s.contains("\"x\""));
}

#[test]
fn tpl_dump_conflict_requires_force() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let mut c = Command::new(exe());
    let o = c.env("J_CONFIG", &cfg).args([":tpl-dump", "d3", "exist"]).output().unwrap();
    assert_ne!(o.status.code().unwrap(), 0);

    let mut c2 = Command::new(exe());
    let o2 = c2.env("J_CONFIG", &cfg).args([":tpl-dump", "--force", "d3", "exist"]).output().unwrap();
    assert_eq!(o2.status.code().unwrap(), 0);
}

#[test]
fn tpl_apply_to_root_is_idempotent() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());

    let mut c = Command::new(exe());
    let o = c.env("J_CONFIG", &cfg).args([":tpl-apply", "d3", "exist"]).output().unwrap();
    assert_eq!(o.status.code().unwrap(), 0, "stderr={}", String::from_utf8_lossy(&o.stderr));

    let mut c2 = Command::new(exe());
    let o2 = c2.env("J_CONFIG", &cfg).args([":tpl-apply", "d3", "exist"]).output().unwrap();
    assert_eq!(o2.status.code().unwrap(), 0, "stderr={}", String::from_utf8_lossy(&o2.stderr));

    let s = fs::read_to_string(&cfg).unwrap();
    assert_eq!(s.matches(r#""templates": ["exist"]"#).count(), 1, "config should not duplicate template refs: {s}");
}

#[test]
fn tpl_apply_to_literal_child() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());

    let mut c = Command::new(exe());
    let o = c.env("J_CONFIG", &cfg).args([":tpl-apply", "d3", "notes", "exist"]).output().unwrap();
    assert_eq!(o.status.code().unwrap(), 0, "stderr={}", String::from_utf8_lossy(&o.stderr));

    let s = fs::read_to_string(&cfg).unwrap();
    assert!(s.contains(r#""notes": {"#), "got: {s}");
    assert!(s.contains(r#""path": "notes""#), "got: {s}");
    assert!(s.contains(r#""templates": ["exist"]"#), "got: {s}");
}

#[test]
fn tpl_apply_rejects_non_literal_target() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());

    let mut c = Command::new(exe());
    let o = c.env("J_CONFIG", &cfg).args([":tpl-apply", "d3", "x", "exist"]).output().unwrap();
    assert_ne!(o.status.code().unwrap(), 0);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("unknown-symbol"), "stderr={err}");
}

#[test]
fn tpl_rm_referenced_requires_force() {
    let dir = tempdir().unwrap();
    let cfg = seed(dir.path());
    let mut c = Command::new(exe());
    let o = c.env("J_CONFIG", &cfg).args([":tpl-rm", "exist"]).output().unwrap();
    assert_ne!(o.status.code().unwrap(), 0);

    let mut c2 = Command::new(exe());
    let o2 = c2.env("J_CONFIG", &cfg).args([":tpl-rm", "--force", "exist"]).output().unwrap();
    assert_eq!(o2.status.code().unwrap(), 0);
    let s = fs::read_to_string(&cfg).unwrap();
    assert!(!s.contains("\"exist\""));
}
