use std::process::Command;

#[allow(dead_code)]
pub fn exe() -> std::path::PathBuf {
    env!("CARGO_BIN_EXE_j").into()
}

#[allow(dead_code)]
pub fn run(args: &[&str]) -> (String, String, i32) {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/config.jsonc");
    let mut c = Command::new(exe());
    c.env("J_CONFIG", fixture);
    c.args(args);
    let o = c.output().unwrap();
    (
        String::from_utf8(o.stdout).unwrap(),
        String::from_utf8(o.stderr).unwrap(),
        o.status.code().unwrap_or(-1),
    )
}
