use j::config::{load_from_str, load_from_str_validated};

const FIXTURE: &str = include_str!("fixtures/config.jsonc");

#[test]
fn load_fixture_ok() {
    let c = load_from_str(FIXTURE, "fixture.jsonc").expect("should load");
    assert_eq!(c.commands.get("c").unwrap(), "code");
    assert_eq!(c.roots.get("d3").unwrap().path, "C:\\projects\\d3");
    assert_eq!(c.roots.get("d3").unwrap().templates, vec!["uProject".to_string()]);
    assert_eq!(c.templates.get("uProject").unwrap()
               .children.get("src").unwrap()
               .children.get("pri").unwrap().path, "Private");
}

#[test]
fn load_missing_roots_is_config_invalid() {
    use j::error::JError;
    let err = load_from_str("{}", "x.jsonc").unwrap_err();
    assert!(matches!(err, JError::ConfigInvalid { .. }));
}

#[test]
fn load_bad_json_is_config_error() {
    use j::error::JError;
    let err = load_from_str("{ not json", "x.jsonc").unwrap_err();
    assert!(matches!(err, JError::ConfigError { .. }));
}

#[test]
fn rejects_relative_root_path() {
    use j::error::JError;
    let src = r#"{"roots":{"d":{"path":"relative\\path"}}}"#;
    let err = load_from_str_validated(src, "x.jsonc").unwrap_err();
    assert!(matches!(err, JError::ConfigInvalid { .. }));
}

#[test]
fn rejects_template_ref_to_unknown() {
    use j::error::JError;
    let src = r#"{"roots":{"d":{"path":"C:\\x","templates":["missing"]}}}"#;
    let err = load_from_str_validated(src, "x.jsonc").unwrap_err();
    assert!(matches!(err, JError::ConfigInvalid { .. }));
}

#[test]
fn rejects_illegal_symbol_name() {
    use j::error::JError;
    let src = r#"{"roots":{"-bad":{"path":"C:\\x"}}}"#;
    let err = load_from_str_validated(src, "x.jsonc").unwrap_err();
    assert!(matches!(err, JError::ConfigInvalid { .. }));
}

#[test]
fn accepts_fixture() {
    let fixture = include_str!("fixtures/config.jsonc");
    load_from_str_validated(fixture, "fixture.jsonc").expect("ok");
}
