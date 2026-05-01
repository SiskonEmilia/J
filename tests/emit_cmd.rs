use j::emit::cmd::emit;

#[test]
fn only_cd() {
    let out = emit("C:\\projects\\d3", None);
    assert_eq!(out.trim(), "cd /d \"C:\\projects\\d3\"");
}

#[test]
fn cd_plus_command() {
    let out = emit("C:\\d3\\Data", Some(&["code".into(), ".".into(), "--new-window".into()]));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "cd /d \"C:\\d3\\Data\"");
    assert_eq!(lines[1], "\"code\" \".\" \"--new-window\"");
}

#[test]
fn escapes_double_quote() {
    let out = emit("C:\\x", Some(&["echo".into(), "say \"hi\"".into()]));
    let last = out.lines().last().unwrap();
    assert_eq!(last, "\"echo\" \"say \"\"hi\"\"\"");
}

#[test]
fn escapes_percent() {
    let out = emit("C:\\x", Some(&["echo".into(), "50%".into()]));
    let last = out.lines().last().unwrap();
    assert_eq!(last, "\"echo\" \"50%%\"");
}
