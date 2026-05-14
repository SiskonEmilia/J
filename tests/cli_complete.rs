mod common;
use common::{exe, run};
use std::process::Command;
use tempfile::tempdir;

#[test]
fn complete_rich_tsv_format() {
    // `:complete-rich` must emit lines of exactly "symbol\tpath"
    let line = "j d3 ";
    let cursor = line.len().to_string();
    let (out, err, code) = run(&[":complete-rich", "powershell", &cursor, line]);
    assert_eq!(code, 0, "stderr={}", err);

    // Every non-empty output line must contain exactly one tab character.
    for l in out.lines().filter(|l| !l.is_empty()) {
        let tab_count = l.chars().filter(|&c| c == '\t').count();
        assert_eq!(tab_count, 1, "line has wrong tab count: {:?}", l);
    }

    // For "j d3 " (cursor=5) the child symbol "notes" resolves to path "docs/notes".
    assert!(
        out.lines().any(|l| l == "notes\tdocs/notes"),
        "expected 'notes\\tdocs/notes' in output, got:\n{}",
        out
    );
}

#[test]
fn complete_roots() {
    let (out, err, code) = run(&[":complete", "powershell", "4", "j d"]);
    assert_eq!(code, 0, "stderr={}", err);
    assert!(out.lines().any(|l| l == "d3"), "got: {}", out);
}

#[test]
fn complete_subcommands() {
    let (out, _err, code) = run(&[":complete", "powershell", "3", "j :"]);
    assert_eq!(code, 0);
    assert!(out.lines().any(|l| l == ":list"));
}

#[test]
fn complete_rich_sanitizes_newlines_and_tabs_in_values() {
    // Alias commands may contain \n/\t; those must not break the TSV record.
    let dir = tempdir().unwrap();
    let cfg = dir.path().join("config.jsonc");
    std::fs::write(
        &cfg,
        r#"{
  "commands": {
    "multi": "echo one\nhelp\ttwo"
  },
  "roots": {
    "r": { "path": "C:\\tmp" }
  }
}"#,
    )
    .unwrap();

    // After "j r -" we expect the alias completion ("-multi", "echo one\nhelp\ttwo").
    let line = "j r -";
    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg).args([
        ":complete-rich",
        "powershell",
        &line.len().to_string(),
        line,
    ]);
    let o = c.output().unwrap();
    assert_eq!(
        o.status.code().unwrap(),
        0,
        "stderr={}",
        String::from_utf8_lossy(&o.stderr)
    );

    let out = String::from_utf8(o.stdout).unwrap();
    // Exactly one record line plus the trailing newline from format!("…\n").
    let records: Vec<&str> = out.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(
        records.len(),
        1,
        "sanitization must collapse newlines in value; got:\n{}",
        out
    );
    let rec = records[0];
    assert_eq!(
        rec.chars().filter(|&c| c == '\t').count(),
        1,
        "record must contain exactly one tab (field separator): {:?}",
        rec
    );
    assert!(rec.starts_with("-multi\t"), "record: {:?}", rec);
    // The alias body lost its structural whitespace but its visible characters remain.
    assert!(rec.contains("echo one"), "record: {:?}", rec);
    assert!(rec.contains("two"), "record: {:?}", rec);
}

#[test]
fn complete_add_path_under_resolved_symbol_path() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("root");
    std::fs::create_dir_all(root.join("docs").join("notes").join("done")).unwrap();
    std::fs::create_dir_all(root.join("docs").join("notes").join("drafts")).unwrap();
    let child_path = format!("docs{}notes", std::path::MAIN_SEPARATOR);
    let cfg = dir.path().join("config.jsonc");
    std::fs::write(
        &cfg,
        format!(
            r#"{{
  "roots": {{
    "r": {{
      "path": "{}",
      "children": {{
        "notes": {{ "path": "{}" }}
      }}
    }}
  }}
}}"#,
            root.display().to_string().replace('\\', "\\\\"),
            child_path.replace('\\', "\\\\")
        ),
    )
    .unwrap();

    let line = "j :add r notes d";
    let mut c = Command::new(exe());
    c.env("J_CONFIG", &cfg)
        .args([":complete", "powershell", &line.len().to_string(), line]);
    let o = c.output().unwrap();
    assert_eq!(
        o.status.code().unwrap(),
        0,
        "stderr={}",
        String::from_utf8_lossy(&o.stderr)
    );

    let out = String::from_utf8(o.stdout).unwrap();
    let done = format!("done{}", std::path::MAIN_SEPARATOR);
    let drafts = format!("drafts{}", std::path::MAIN_SEPARATOR);
    assert!(out.lines().any(|l| l == done), "got: {out}");
    assert!(out.lines().any(|l| l == drafts), "got: {out}");
}
