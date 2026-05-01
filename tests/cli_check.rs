mod common;
use common::exe;
use std::process::Command;
use tempfile::tempdir;
use std::fs;

#[test]
fn check_reports_missing_paths() {
    let dir = tempdir().unwrap();
    let cfg = dir.path().join("config.jsonc");
    fs::write(&cfg, r#"{
        "roots": {
            "r": {
                "path": "C:\\does\\not\\exist\\hopefully",
                "children": {
                    "a": { "path": "A" }
                }
            }
        }
    }"#).unwrap();

    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).arg(":check");
    let o = c.output().unwrap();
    let stderr = String::from_utf8(o.stderr).unwrap();
    assert_ne!(o.status.code().unwrap(), 0, "expected non-zero");
    assert!(stderr.contains("missing"), "stderr={}", stderr);
}

#[test]
fn check_ok_on_existing_paths() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("root");
    fs::create_dir_all(root.join("sub")).unwrap();
    let cfg = dir.path().join("config.jsonc");
    fs::write(&cfg, format!(r#"{{"roots":{{"r":{{"path":{:?},"children":{{"a":{{"path":"sub"}}}}}}}}}}"#, root.display().to_string().replace('\\', "\\\\"))).unwrap();

    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).arg(":check");
    let o = c.output().unwrap();
    assert_eq!(o.status.code().unwrap(), 0, "stderr={:?}", String::from_utf8_lossy(&o.stderr));
}
