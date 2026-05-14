use j::config::load_from_str_validated;
use j::error::JError;
use j::resolve::{resolve, resolve_current_alias};

const FIXTURE: &str = include_str!("fixtures/config.jsonc");

fn load() -> j::model::Config {
    load_from_str_validated(FIXTURE, "x.jsonc").unwrap()
}

#[test]
fn jump_root_only() {
    let r = resolve(&["d3"], None, &[], &load()).unwrap();
    assert_eq!(r.abs_path, "C:\\projects\\d3");
    assert!(r.post_argv.is_none());
}

#[test]
fn jump_symbol_from_template() {
    let r = resolve(&["d3", "d"], None, &[], &load()).unwrap();
    assert!(r.abs_path.replace('/', "\\").ends_with("d3\\Data"));
}

#[test]
fn jump_symbol_self_overrides() {
    let r = resolve(&["d3", "notes"], None, &[], &load()).unwrap();
    assert!(r.abs_path.replace('/', "\\").ends_with("d3\\docs\\notes"));
}

#[test]
fn jump_nested_template_symbol() {
    let r = resolve(&["d3", "src", "pri"], None, &[], &load()).unwrap();
    assert!(r
        .abs_path
        .replace('/', "\\")
        .ends_with("d3\\Source\\Private"));
}

#[test]
fn jump_with_alias_no_dot() {
    let r = resolve(&["d3", "d"], Some("c"), &[], &load()).unwrap();
    assert_eq!(r.post_argv.as_ref().unwrap(), &["code"]);
}

#[test]
fn jump_with_alias_multi_word_tokenized() {
    let r = resolve(&["d3", "d"], Some("g"), &[], &load()).unwrap();
    assert_eq!(r.post_argv.as_ref().unwrap(), &["git", "status"]);
}

#[test]
fn jump_alias_pass_through_args() {
    let r = resolve(&["d3", "d"], Some("c"), &["--new-window".into()], &load()).unwrap();
    assert_eq!(r.post_argv.as_ref().unwrap(), &["code", "--new-window"]);
}

#[test]
fn current_dir_alias_uses_supplied_directory() {
    let r =
        resolve_current_alias("c", &["--new-window".into()], &load(), "C:\\work".into()).unwrap();
    assert_eq!(r.abs_path, "C:\\work");
    assert_eq!(r.post_argv.as_ref().unwrap(), &["code", "--new-window"]);
}

#[test]
fn unknown_root() {
    let err = resolve(&["d9"], None, &[], &load()).unwrap_err();
    assert!(matches!(err, JError::UnknownRoot { .. }));
}

#[test]
fn unknown_symbol() {
    let err = resolve(&["d3", "zzz"], None, &[], &load()).unwrap_err();
    assert!(matches!(err, JError::UnknownSymbol { .. }));
}

#[test]
fn unknown_alias() {
    let err = resolve(&["d3"], Some("nope"), &[], &load()).unwrap_err();
    assert!(matches!(err, JError::UnknownAlias { .. }));
}

#[test]
fn jump_with_alias_double_quoted() {
    let r = resolve(&["d3"], Some("vsc"), &[], &load()).unwrap();
    assert_eq!(r.post_argv.as_ref().unwrap(), &["open", "-a", "Visual Studio Code"]);
}

#[test]
fn jump_with_alias_single_quoted() {
    let r = resolve(&["d3"], Some("sq"), &[], &load()).unwrap();
    assert_eq!(r.post_argv.as_ref().unwrap(), &["echo", "hello world"]);
}

#[test]
fn jump_with_alias_backslash_escaped() {
    let r = resolve(&["d3"], Some("bse"), &[], &load()).unwrap();
    assert_eq!(r.post_argv.as_ref().unwrap(), &["a b"]);
}

#[test]
fn unknown_current_dir_alias() {
    let err = resolve_current_alias("nope", &[], &load(), "C:\\work".into()).unwrap_err();
    assert!(matches!(err, JError::UnknownAlias { .. }));
}
