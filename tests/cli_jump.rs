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
    assert_eq!(
        lines[0],
        format!("Set-Location -LiteralPath '{}'", dir.path().display())
    );
    assert_eq!(lines[1], "& 'code' '--new-window'");
}

#[test]
fn unknown_root_stderr() {
    let (out, err, code) = run(&["--shell=powershell", "nope"]);
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert!(err.starts_with("j: unknown-root:"), "stderr={}", err);
}
