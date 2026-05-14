use j::cli::{parse, Invocation, Shell};

#[test]
fn jump_root_only() {
    let v = parse(&["d3"]).unwrap();
    match v {
        Invocation::Jump {
            positional,
            alias,
            alias_args,
            shell,
        } => {
            assert_eq!(positional, vec!["d3"]);
            assert!(alias.is_none());
            assert!(alias_args.is_empty());
            assert!(shell.is_none());
        }
        _ => panic!("expected Jump"),
    }
}

#[test]
fn jump_with_alias_and_passthrough() {
    let v = parse(&["d3", "d", "-c", "--new-window", "foo"]).unwrap();
    match v {
        Invocation::Jump {
            positional,
            alias,
            alias_args,
            ..
        } => {
            assert_eq!(positional, vec!["d3", "d"]);
            assert_eq!(alias.as_deref(), Some("c"));
            assert_eq!(alias_args, vec!["--new-window", "foo"]);
        }
        _ => panic!(),
    }
}

#[test]
fn current_dir_alias() {
    let v = parse(&["-c", "--new-window", "foo"]).unwrap();
    match v {
        Invocation::Jump {
            positional,
            alias,
            alias_args,
            ..
        } => {
            assert!(positional.is_empty());
            assert_eq!(alias.as_deref(), Some("c"));
            assert_eq!(alias_args, vec!["--new-window", "foo"]);
        }
        _ => panic!(),
    }
}

#[test]
fn shell_flag_strips() {
    let v = parse(&["--shell=powershell", "d3"]).unwrap();
    match v {
        Invocation::Jump {
            positional, shell, ..
        } => {
            assert_eq!(positional, vec!["d3"]);
            assert_eq!(shell, Some(Shell::PowerShell));
        }
        _ => panic!(),
    }
}

#[test]
fn zsh_shell_flag_is_posix() {
    let v = parse(&["--shell=zsh", "d3"]).unwrap();
    match v {
        Invocation::Jump {
            positional, shell, ..
        } => {
            assert_eq!(positional, vec!["d3"]);
            assert_eq!(shell, Some(Shell::Posix));
        }
        _ => panic!(),
    }
}

#[test]
fn subcommand_colon_prefix() {
    let v = parse(&[":list", "d3"]).unwrap();
    assert!(matches!(v, Invocation::Subcmd { name, args } if name == "list" && args == vec!["d3"]));
}

#[test]
fn double_dash_help_alias() {
    let v = parse(&["--help"]).unwrap();
    assert!(matches!(v, Invocation::Subcmd { ref name, .. } if name == "help"));
}

#[test]
fn no_args_is_help() {
    let v = parse(&[]).unwrap();
    assert!(matches!(v, Invocation::Subcmd { ref name, .. } if name == "help"));
}
