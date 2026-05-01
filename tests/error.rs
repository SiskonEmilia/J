use j::error::{JError, exit_code};

#[test]
fn unknown_root_maps_to_code_2() {
    let e = JError::UnknownRoot {
        name: "d5".into(),
        available: vec!["d3".into(), "d4".into()],
    };
    assert_eq!(exit_code(&e), 2);
}

#[test]
fn config_error_maps_to_code_3() {
    let e = JError::ConfigError {
        path: "x.jsonc".into(),
        line: 17,
        col: 5,
        msg: "expected ',' or '}'".into(),
    };
    assert_eq!(exit_code(&e), 3);
}
