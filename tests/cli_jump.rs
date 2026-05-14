mod common;
use common::{exe, run};
use std::process::Command;
use tempfile::tempdir;

#[test]
fn jump_root_ps() {
    let (out, err, code) = run(&["--shell=powershell", "d3"]);
    assert_eq!(code, 0, "stderr={}", err);
    insta::assert_snapshot!(out, @r###"
    Set-Location -LiteralPath 'C:\projects\d3'
    "###);
}

#[test]
fn jump_symbol_with_alias_ps() {
    let (out, err, code) = run(&["--shell=powershell", "d3", "d", "-c", "--new-window"]);
    assert_eq!(code, 0, "stderr={}", err);
    insta::assert_snapshot!(out);
}

#[test]
fn jump_cmd_emit() {
    let (out, err, code) = run(&["--shell=cmd", "d3", "d", "-cc"]);
    assert_eq!(code, 0, "stderr={}", err);
    insta::assert_snapshot!(out);
}

#[test]
fn jump_zsh_emit() {
    let (out, err, code) = run(&["--shell=zsh", "d3", "d", "-c", "--new-window"]);
    assert_eq!(code, 0, "stderr={}", err);
    let normalized = out.replace('\\', "/");
    assert!(
        normalized.contains("cd -- 'C:/projects/d3/Data'"),
        "out={out}"
    );
    assert!(out.contains("'code' '--new-window'"), "out={out}");
}

#[test]
fn current_dir_alias_ps() {
    let dir = tempdir().unwrap();
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/config.jsonc");
    let mut c = Command::new(exe());
    c.env("J_CONFIG", fixture).current_dir(dir.path()).args([
        "--shell=powershell",
        "-c",
        "--new-window",
    ]);
    let o = c.output().unwrap();
    assert_eq!(
        o.status.code().unwrap(),
        0,
        "stderr={}",
        String::from_utf8_lossy(&o.stderr)
    );

    let out = String::from_utf8(o.stdout).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    let prefix = "Set-Location -LiteralPath '";
    let suffix = "'";
    let actual_path_str = lines[0]
        .strip_prefix(prefix)
        .and_then(|s| s.strip_suffix(suffix))
        .expect("cd line format");
    let actual_canon = std::fs::canonicalize(actual_path_str)
        .unwrap_or_else(|_| std::path::PathBuf::from(actual_path_str));
    let expected_canon = std::fs::canonicalize(dir.path()).unwrap();
    assert_eq!(actual_canon, expected_canon);
    assert_eq!(lines[1], "& 'code' '--new-window'");
}

#[test]
fn unknown_root_stderr() {
    let (out, err, code) = run(&["--shell=powershell", "nope"]);
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert!(err.starts_with("j: unknown-root:"), "stderr={}", err);
}
