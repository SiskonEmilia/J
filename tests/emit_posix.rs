use j::emit::posix::emit;

#[test]
fn posix_cd_only() {
    assert_eq!(emit("/Users/me/work", None), "cd -- '/Users/me/work'\n");
}

#[test]
fn posix_quotes_single_quotes_and_alias_args() {
    let argv = vec!["code".to_string(), "Bob's Project".to_string()];
    assert_eq!(
        emit("/Users/me/Bob's Project", Some(&argv)),
        "cd -- '/Users/me/Bob'\\''s Project'\n'code' 'Bob'\\''s Project'\n"
    );
}
