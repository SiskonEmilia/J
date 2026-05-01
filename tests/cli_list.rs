mod common;
use common::run;

#[test]
fn list_all() {
    let (out, err, code) = run(&[":list"]);
    assert_eq!(code, 0, "stderr={}", err);
    insta::assert_snapshot!(out);
}

#[test]
fn list_root() {
    let (out, err, code) = run(&[":list", "d3"]);
    assert_eq!(code, 0, "stderr={}", err);
    insta::assert_snapshot!(out);
}

#[test]
fn list_subtree() {
    let (out, err, code) = run(&[":list", "d3", "src"]);
    assert_eq!(code, 0, "stderr={}", err);
    insta::assert_snapshot!(out);
}
