mod common;
use common::run;

#[test]
fn config_path_prints_env() {
    let (out, err, code) = run(&[":config-path"]);
    assert_eq!(code, 0, "stderr={}", err);
    assert!(out.trim().ends_with("config.jsonc"), "got: {}", out);
}

#[test]
fn help_contains_jump_usage() {
    let (out, err, code) = run(&[":help"]);
    assert_eq!(code, 0, "stderr={}", err);
    assert!(out.contains("j <root>"), "got: {}", out);
    assert!(out.contains("j -<alias>"), "got: {}", out);
    assert!(out.contains(":tpl-apply"), "got: {}", out);
}

#[test]
fn version_prints_semver() {
    let (out, err, code) = run(&[":version"]);
    assert_eq!(code, 0, "stderr={}", err);
    assert!(out.contains(env!("CARGO_PKG_VERSION")), "got: {}", out);
}
