use j::install::posix::{build_bash_completion, build_shim_script, install_into_file, uninstall_from_file};
use j::subcmd::install;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn posix_shim_invokes_zsh_shell_mode() {
    let s = build_shim_script("/usr/local/bin/j", "zsh");
    assert!(s.contains("--shell=zsh"));
    assert!(s.contains("eval \"$_j_out\""));
}

#[test]
fn install_idempotent() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join(".zshrc");

    install_into_file(&profile, "/usr/local/bin/j", "zsh").unwrap();
    install_into_file(&profile, "/usr/local/bin/j", "zsh").unwrap();

    let s = std::fs::read_to_string(&profile).unwrap();
    assert_eq!(s.matches("# region j-shim").count(), 1);
    assert_eq!(s.matches("# endregion j-shim").count(), 1);
}

#[test]
fn uninstall_removes_region() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join(".zshrc");

    install_into_file(&profile, "/usr/local/bin/j", "zsh").unwrap();
    uninstall_from_file(&profile).unwrap();

    let s = std::fs::read_to_string(&profile).unwrap();
    assert!(!s.contains("BEGIN J"));
    assert!(!s.contains("j()"));
}

#[test]
fn custom_profile_install_and_uninstall() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join("my_custom_profile.sh");
    let profile_s = profile.to_str().unwrap().to_string();

    let args: Vec<String> = vec![
        "zsh".to_string(),
        "--profile".to_string(),
        profile_s.clone(),
    ];

    let result = install::install(Path::new(""), &args).unwrap();
    assert!(result.contains("installed zsh shim into:"));
    assert!(result.contains(&profile_s));

    let s = std::fs::read_to_string(&profile).unwrap();
    assert!(s.contains("# region j-shim"));
    assert!(s.contains("# endregion j-shim"));

    install::uninstall(Path::new(""), &args).unwrap();

    let s = std::fs::read_to_string(&profile).unwrap();
    assert!(!s.contains("# region j-shim"));
}

#[test]
fn custom_profile_repeat_install_is_idempotent() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join("my_profile.sh");
    let profile_s = profile.to_str().unwrap().to_string();

    let args: Vec<String> = vec![
        "zsh".to_string(),
        "--profile".to_string(),
        profile_s.clone(),
    ];

    install::install(Path::new(""), &args).unwrap();
    install::install(Path::new(""), &args).unwrap();

    let s = std::fs::read_to_string(&profile).unwrap();
    assert_eq!(s.matches("# region j-shim").count(), 1);
    assert_eq!(s.matches("# endregion j-shim").count(), 1);
}

#[test]
fn uninstall_no_op_when_region_absent() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join("no_region.sh");
    let profile_s = profile.to_str().unwrap().to_string();

    std::fs::write(&profile, "echo hello\n").unwrap();

    let args: Vec<String> = vec![
        "zsh".to_string(),
        "--profile".to_string(),
        profile_s.clone(),
    ];

    install::uninstall(Path::new(""), &args).unwrap();

    let s = std::fs::read_to_string(&profile).unwrap();
    assert_eq!(s, "echo hello\n");
}

#[test]
fn profile_flag_parsed_for_bash() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join("bashrc_custom");
    let profile_s = profile.to_str().unwrap().to_string();

    let args: Vec<String> = vec![
        "bash".to_string(),
        "--profile".to_string(),
        profile_s.clone(),
    ];

    let result = install::install(Path::new(""), &args).unwrap();
    assert!(result.contains("installed bash shim into:"));
    assert!(result.contains(&profile_s));

    let s = std::fs::read_to_string(&profile).unwrap();
    assert!(s.contains("j()"));
}

#[test]
fn profile_flag_parsed_for_sh() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join("myprofile.sh");
    let profile_s = profile.to_str().unwrap().to_string();

    let args: Vec<String> = vec![
        "sh".to_string(),
        "--profile".to_string(),
        profile_s.clone(),
    ];

    let result = install::install(Path::new(""), &args).unwrap();
    assert!(result.contains("installed sh shim into:"));
    assert!(result.contains(&profile_s));

    let s = std::fs::read_to_string(&profile).unwrap();
    assert!(s.contains("j()"));
    assert!(s.contains("--shell=sh"));
}

#[test]
fn zsh_shim_contains_completion_function() {
    let s = build_shim_script("/usr/local/bin/j", "zsh");
    assert!(s.contains("_j()"));
    assert!(s.contains(":complete zsh"));
    assert!(s.contains("compadd -Q"));
}

#[test]
fn zsh_shim_contains_compdef() {
    let s = build_shim_script("/usr/local/bin/j", "zsh");
    assert!(s.contains("compdef _j j"));
}

#[test]
fn bash_shim_does_not_contain_zsh_completion() {
    let s = build_shim_script("/usr/local/bin/j", "bash");
    assert!(!s.contains("_j()"));
    assert!(!s.contains("compdef _j j"));
    assert!(!s.contains("compadd"));
}

#[test]
fn sh_shim_does_not_contain_completion() {
    let s = build_shim_script("/usr/local/bin/j", "sh");
    assert!(!s.contains("_j()"));
    assert!(!s.contains("compdef _j j"));
    assert!(!s.contains("compadd"));
}

#[test]
fn bash_shim_contains_completion_function() {
    let s = build_shim_script("/usr/local/bin/j", "bash");
    assert!(s.contains("_j_complete_bash()"));
}

#[test]
fn bash_shim_contains_register_complete() {
    let s = build_shim_script("/usr/local/bin/j", "bash");
    assert!(s.contains("complete -F _j_complete_bash j 2>/dev/null || true"));
}

#[test]
fn sh_shim_does_not_contain_bash_completion() {
    let s = build_shim_script("/usr/local/bin/j", "sh");
    assert!(!s.contains("_j_complete_bash"));
    assert!(!s.contains("complete -F _j_complete_bash j"));
}

#[test]
fn zsh_shim_does_not_contain_bash_completion() {
    let s = build_shim_script("/usr/local/bin/j", "zsh");
    assert!(!s.contains("_j_complete_bash"));
    assert!(!s.contains("complete -F _j_complete_bash j"));
}

#[test]
fn build_bash_completion_generates_function() {
    let s = build_bash_completion("/usr/local/bin/j");
    assert!(s.contains("_j_complete_bash()"));
    assert!(s.contains(":complete bash"));
    assert!(s.contains("$COMP_POINT"));
    assert!(s.contains("$COMP_LINE"));
    assert!(s.contains("IFS=$'\\n'"));
}

#[test]
#[cfg(unix)]
fn default_profile_when_no_flag() {
    let args: Vec<String> = vec!["zsh".to_string()];

    install::install(Path::new(""), &args).unwrap();

    let Ok(home) = std::env::var("HOME") else { return };
    let default = std::path::PathBuf::from(&home).join(".zshrc");
    if default.exists() {
        let s = std::fs::read_to_string(&default).unwrap();
        assert!(s.contains("# region j-shim"));

        install::uninstall(Path::new(""), &args).unwrap();

        let s2 = std::fs::read_to_string(&default).unwrap();
        assert!(!s2.contains("# region j-shim"));
    }
}
