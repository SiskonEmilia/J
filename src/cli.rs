use crate::error::JError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell { PowerShell, Cmd }

#[derive(Debug)]
pub enum Invocation {
    Jump {
        positional: Vec<String>,
        alias: Option<String>,
        alias_args: Vec<String>,
        shell: Option<Shell>,
    },
    Subcmd {
        name: String,
        args: Vec<String>,
    },
}

pub fn parse(argv: &[&str]) -> Result<Invocation, JError> {
    // 1) strip global flags (--shell=X, --help, --version, -h)
    let mut shell: Option<Shell> = None;
    let mut rest: Vec<String> = Vec::new();
    for a in argv {
        if let Some(v) = a.strip_prefix("--shell=") {
            shell = Some(match v {
                "powershell" => Shell::PowerShell,
                "cmd"        => Shell::Cmd,
                _ => return Err(JError::Internal { msg: format!("unknown --shell value '{}'", v) }),
            });
        } else if *a == "--help" || *a == "-h" {
            return Ok(Invocation::Subcmd { name: "help".into(), args: Vec::new() });
        } else if *a == "--version" {
            return Ok(Invocation::Subcmd { name: "version".into(), args: Vec::new() });
        } else {
            rest.push((*a).to_string());
        }
    }

    if rest.is_empty() {
        return Ok(Invocation::Subcmd { name: "help".into(), args: Vec::new() });
    }

    // 2) subcommand if first token starts with ':'
    if let Some(cmd) = rest[0].strip_prefix(':') {
        return Ok(Invocation::Subcmd {
            name: cmd.to_string(),
            args: rest[1..].to_vec(),
        });
    }

    // 3) otherwise Jump: scan for -<alias>
    let mut positional: Vec<String> = Vec::new();
    let mut alias: Option<String> = None;
    let mut alias_args: Vec<String> = Vec::new();
    let mut in_alias = false;
    for t in rest {
        if in_alias {
            alias_args.push(t);
        } else if let Some(name) = t.strip_prefix('-') {
            alias = Some(name.to_string());
            in_alias = true;
        } else {
            positional.push(t);
        }
    }

    Ok(Invocation::Jump { positional, alias, alias_args, shell })
}
