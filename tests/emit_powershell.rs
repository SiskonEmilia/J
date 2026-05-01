use j::emit::powershell::emit;

#[test]
fn only_cd() {
    let out = emit("C:\\projects\\d3", None);
    assert_eq!(out.trim(), "Set-Location -LiteralPath 'C:\\projects\\d3'");
}

#[test]
fn cd_plus_command() {
    let out = emit("C:\\projects\\d3\\Data",
        Some(&["code".into(), ".".into(), "--new-window".into()]));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "Set-Location -LiteralPath 'C:\\projects\\d3\\Data'");
    assert_eq!(lines[1], "& 'code' '.' '--new-window'");
}

#[test]
fn escapes_single_quote_in_path() {
    let out = emit("C:\\it's\\fine", None);
    assert_eq!(out.trim(), "Set-Location -LiteralPath 'C:\\it''s\\fine'");
}

#[test]
fn escapes_single_quote_in_arg() {
    let out = emit("C:\\x", Some(&["echo".into(), "it's".into()]));
    let last = out.lines().last().unwrap();
    assert_eq!(last, "& 'echo' 'it''s'");
}
