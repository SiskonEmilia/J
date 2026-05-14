pub fn emit(abs_path: &str, post_argv: Option<&[String]>) -> String {
    let mut out = String::new();
    out.push_str(&format!("cd -- {}\n", quote(abs_path)));
    if let Some(argv) = post_argv {
        if !argv.is_empty() {
            for (i, a) in argv.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                out.push_str(&quote(a));
            }
            out.push('\n');
        }
    }
    out
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}
