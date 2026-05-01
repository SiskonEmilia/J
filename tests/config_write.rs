use j::config::write::load_cst;

const SRC: &str = r#"{
  // keep me
  "commands": {
    "c": "code"    // alias c
  },
  "roots": {
    "d3": { "path": "C:\\d3" }
  }
}
"#;

#[test]
fn roundtrip_preserves_text() {
    let doc = load_cst(SRC).unwrap();
    assert_eq!(doc.to_string(), SRC);
}

#[test]
fn add_root_preserves_comments() {
    let mut doc = load_cst(SRC).unwrap();
    doc.upsert_root("d4", "D:\\d4").unwrap();
    let out = doc.to_string();
    assert!(out.contains("// keep me"));
    assert!(out.contains("// alias c"));
    assert!(out.contains("\"d4\""));
    assert!(out.contains("D:\\\\d4"));
}

#[test]
fn set_alias_roundtrip() {
    let mut doc = load_cst(SRC).unwrap();
    doc.set_alias("g", "git status").unwrap();
    let out = doc.to_string();
    assert!(out.contains("\"g\""));
    assert!(out.contains("git status"));
    assert!(out.contains("// keep me"));
}

#[test]
fn remove_root() {
    let mut doc = load_cst(SRC).unwrap();
    doc.remove_root("d3").unwrap();
    let out = doc.to_string();
    assert!(!out.contains("\"d3\""));
    assert!(out.contains("\"commands\""));
}
