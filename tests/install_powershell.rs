use j::install::powershell::{all_profile_paths, build_shim_script, install_into_file, uninstall_from_file};
use tempfile::tempdir;
use std::fs;

#[test]
fn shim_script_references_absolute_exe() {
    let body = build_shim_script("C:\\tools\\j\\j.exe");
    assert!(body.contains("'C:\\tools\\j\\j.exe'"));
    assert!(body.contains("function j"));
    assert!(body.contains("Invoke-Expression"));
    assert!(body.contains("Register-ArgumentCompleter"));
    assert!(!body.contains("-Native"), "PowerShell completer should not fall back to native filesystem completion");
    assert!(body.contains("$commandAst.ToString()"), "completer must extract command line");
    assert!(body.contains("$ln = $parameterName; $cur = [int]$wordToComplete"), "completer must have PS 5.1 fallback");
    // Smart dispatch: execute only jump scripts; display subcommand text via Write-Host.
    assert!(body.contains("-match '^Set-Location'"), "shim must guard Invoke-Expression with Set-Location check");
    assert!(body.contains("Write-Host"), "shim must display non-script output");
    assert!(body.contains(":complete-rich"), "interactive mode must call :complete-rich");
    assert!(body.contains("ReadKey"), "interactive mode must use ReadKey for input");
    assert!(body.contains("_jBuf"), "interactive mode must maintain input buffer");
}

#[test]
fn install_idempotent() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join("Microsoft.PowerShell_profile.ps1");
    fs::write(&profile, "# existing profile\n$PSModulePath | Out-Null\n").unwrap();

    install_into_file(&profile, "C:\\tools\\j\\j.exe").unwrap();
    let s1 = fs::read_to_string(&profile).unwrap();
    assert!(s1.contains("# region j-shim"));
    assert_eq!(s1.matches("# region j-shim").count(), 1);

    install_into_file(&profile, "C:\\tools\\j\\j.exe").unwrap();
    let s2 = fs::read_to_string(&profile).unwrap();
    assert_eq!(s2.matches("# region j-shim").count(), 1);
    assert!(s2.contains("# existing profile"));
}

#[test]
fn all_profile_paths_covers_both_ps_versions() {
    let paths = all_profile_paths().unwrap();
    let strs: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
    assert!(strs.iter().any(|s| s.contains("WindowsPowerShell")),
        "must include Windows PowerShell 5.1 path");
    assert!(strs.iter().any(|s| s.contains(r"PowerShell\Microsoft")),
        "must include PowerShell 7 path");
}

/// Verifies the emitted shim parses as valid PowerShell. Guards against
/// brace-escaping regressions in the Rust format! template and any future
/// syntax drift. Skipped silently if no PS host is available on PATH.
#[cfg(windows)]
#[test]
fn shim_script_is_valid_powershell_syntax() {
    let ps_host = ["pwsh.exe", "powershell.exe"]
        .into_iter()
        .find(|exe| std::process::Command::new(exe)
            .arg("-NoProfile").arg("-Command").arg("exit 0")
            .output().map(|o| o.status.success()).unwrap_or(false));
    let Some(ps_host) = ps_host else {
        eprintln!("skipping: no PowerShell host on PATH");
        return;
    };

    let body = build_shim_script("C:\\tools\\j\\j.exe");
    let tmp_dir = tempdir().unwrap();
    let script_path = tmp_dir.path().join("shim.ps1");
    fs::write(&script_path, &body).unwrap();

    // Ask the PS host itself to tokenize+parse the file and report any errors.
    let parse_cmd = format!(
        "$errs=$null;[void][System.Management.Automation.Language.Parser]::ParseFile('{}',[ref]$null,[ref]$errs);\
         if($errs -and $errs.Count -gt 0){{foreach($e in $errs){{Write-Error $e.ToString()}};exit 1}}",
        script_path.display().to_string().replace('\'', "''")
    );
    let out = std::process::Command::new(ps_host)
        .args(["-NoProfile", "-Command", &parse_cmd])
        .output()
        .unwrap();
    assert!(out.status.success(),
        "shim failed to parse under {ps_host}:\nstderr={}\nstdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout));
}

#[test]
fn uninstall_removes_region() {
    let dir = tempdir().unwrap();
    let profile = dir.path().join("p.ps1");
    fs::write(&profile, "# A\n").unwrap();
    install_into_file(&profile, "C:\\j\\j.exe").unwrap();
    uninstall_from_file(&profile).unwrap();
    let s = fs::read_to_string(&profile).unwrap();
    assert!(!s.contains("# region j-shim"));
    assert!(s.contains("# A"));
}
