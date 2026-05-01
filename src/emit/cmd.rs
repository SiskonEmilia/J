pub fn emit(abs_path: &str, post_argv: Option<&[String]>) -> String {
    let mut out = String::new();
    out.push_str(&format!("cd /d {}\r\n", quote(abs_path)));
    if let Some(argv) = post_argv {
        if !argv.is_empty() {
            for (i, a) in argv.iter().enumerate() {
                if i > 0 { out.push(' '); }
                out.push_str(&quote(a));
            }
            out.push_str("\r\n");
        }
    }
    out
}

/// cmd 双引号字符串：内部 " 用 "" 转义；% 用 %% 转义（call 到 .bat 时会展开环境变量）
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => { out.push('"'); out.push('"'); }
            '%' => { out.push('%'); out.push('%'); }
            _   => out.push(ch),
        }
    }
    out.push('"');
    out
}
