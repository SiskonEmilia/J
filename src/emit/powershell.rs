pub fn emit(abs_path: &str, post_argv: Option<&[String]>) -> String {
    let mut out = String::new();
    out.push_str(&format!("Set-Location -LiteralPath {}\n", quote(abs_path)));
    if let Some(argv) = post_argv {
        if !argv.is_empty() {
            out.push_str("& ");
            for (i, a) in argv.iter().enumerate() {
                if i > 0 { out.push(' '); }
                out.push_str(&quote(a));
            }
            out.push('\n');
        }
    }
    out
}

/// PowerShell 单引号字符串：内部单引号用 '' 转义；其它字符原样。
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push('\'');
            out.push('\'');
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}
