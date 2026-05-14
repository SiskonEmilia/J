#[derive(Debug)]
pub struct TokenizeError {
    pub pos: usize,
    pub msg: String,
}

pub fn shell_tokenize(cmd: &str) -> Result<Vec<String>, TokenizeError> {
    let chars: Vec<char> = cmd.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_token = false;

    while i < len {
        let ch = chars[i];

        if ch == '\'' {
            in_token = true;
            let start = i;
            i += 1;
            while i < len && chars[i] != '\'' {
                current.push(chars[i]);
                i += 1;
            }
            if i >= len {
                return Err(TokenizeError {
                    pos: start,
                    msg: "unterminated single quote".into(),
                });
            }
            i += 1;
            continue;
        }

        if ch == '"' {
            in_token = true;
            let start = i;
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                    if i < len {
                        match chars[i] {
                            'n' => current.push('\n'),
                            't' => current.push('\t'),
                            other => current.push(other),
                        }
                        i += 1;
                    }
                } else {
                    current.push(chars[i]);
                    i += 1;
                }
            }
            if i >= len {
                return Err(TokenizeError {
                    pos: start,
                    msg: "unterminated double quote".into(),
                });
            }
            i += 1;
            continue;
        }

        if ch == '\\' {
            in_token = true;
            i += 1;
            if i < len {
                current.push(chars[i]);
                i += 1;
            }
            continue;
        }

        if ch.is_whitespace() {
            if in_token {
                tokens.push(std::mem::take(&mut current));
                in_token = false;
            }
            i += 1;
            continue;
        }

        in_token = true;
        current.push(ch);
        i += 1;
    }

    if in_token {
        tokens.push(current);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_words() {
        let r = shell_tokenize("git status").unwrap();
        assert_eq!(r, vec!["git", "status"]);
    }

    #[test]
    fn double_quoted_arg() {
        let r = shell_tokenize(r#"open -a "Visual Studio Code""#).unwrap();
        assert_eq!(r, vec!["open", "-a", "Visual Studio Code"]);
    }

    #[test]
    fn single_quoted_arg() {
        let r = shell_tokenize("echo 'hello world'").unwrap();
        assert_eq!(r, vec!["echo", "hello world"]);
    }

    #[test]
    fn backslash_escaped_space() {
        let r = shell_tokenize(r"a\ b").unwrap();
        assert_eq!(r, vec!["a b"]);
    }

    #[test]
    fn backslash_escaped_backslash() {
        let r = shell_tokenize(r"a\\b").unwrap();
        assert_eq!(r, vec![r"a\b"]);
    }

    #[test]
    fn empty_double_quoted() {
        let r = shell_tokenize(r#"a "" b"#).unwrap();
        assert_eq!(r, vec!["a", "", "b"]);
    }

    #[test]
    fn empty_single_quoted() {
        let r = shell_tokenize("a '' b").unwrap();
        assert_eq!(r, vec!["a", "", "b"]);
    }

    #[test]
    fn double_quote_escapes() {
        let r = shell_tokenize(r#""a\nb\tc\\d\"e""#).unwrap();
        assert_eq!(r, vec!["a\nb\tc\\d\"e"]);
    }

    #[test]
    fn only_whitespace() {
        let r = shell_tokenize("   ").unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn empty_input() {
        let r = shell_tokenize("").unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn unterminated_double_quote() {
        let err = shell_tokenize(r#""hello"#).unwrap_err();
        assert_eq!(err.pos, 0);
        assert!(err.msg.contains("unterminated double quote"));
    }

    #[test]
    fn unterminated_single_quote() {
        let err = shell_tokenize("'hello").unwrap_err();
        assert_eq!(err.pos, 0);
        assert!(err.msg.contains("unterminated single quote"));
    }

    #[test]
    fn adjacent_quoted_sections_merge() {
        let r = shell_tokenize(r#"a"b"c"#).unwrap();
        assert_eq!(r, vec!["abc"]);
    }

    #[test]
    fn mixed_quotes() {
        let r = shell_tokenize(r#"echo "it's fine" 'and "this" too'"#).unwrap();
        assert_eq!(r, vec!["echo", "it's fine", r#"and "this" too"#]);
    }
}
