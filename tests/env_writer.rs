use j::install::{EnvWriter, MockEnvWriter};

#[test]
fn mock_env_writer_appends_to_path() {
    let mock = MockEnvWriter::new("C:\\Windows;C:\\other");
    mock.ensure_path_contains("C:\\tools\\j\\bin").unwrap();
    assert_eq!(mock.read_path().unwrap(), "C:\\Windows;C:\\other;C:\\tools\\j\\bin");
    // idempotent
    mock.ensure_path_contains("C:\\tools\\j\\bin").unwrap();
    assert_eq!(mock.read_path().unwrap(), "C:\\Windows;C:\\other;C:\\tools\\j\\bin");
}

#[test]
fn mock_env_writer_removes_from_path() {
    let mock = MockEnvWriter::new("C:\\Windows;C:\\tools\\j\\bin;C:\\other");
    mock.ensure_path_not_contains("C:\\tools\\j\\bin").unwrap();
    assert_eq!(mock.read_path().unwrap(), "C:\\Windows;C:\\other");
}
