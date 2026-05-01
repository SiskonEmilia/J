use crate::config::load_from_str_validated;
use crate::error::JError;
use std::path::Path;

const HELP: &str = r#"Usage:
  j <root> [<sym>...] [-<alias> [args...]]       jump
  j -<alias> [args...]                            run alias in current directory
  j                                                show help (same as :help)

Subcommands:
  j :list [<root> [<sym>...]]                    show tree (no args = all roots + commands + templates)
  j :add <root> [<sym>...] <path>                add/modify node; path is stored as-is (validated by :check)
  j :add <root> .                                add current directory as root
  j :rm <root> [<sym>...]                        remove node/root
  j :alias <name> <cmd>                          set alias
  j :alias --rm <name>                           remove alias
  j :tpl-dump [--force] <root> [<sym>...] <tpl>  dump root/subtree children to template (--force to overwrite)
  j :tpl-apply <root> [<sym>...] <tpl-name>      attach template to an existing configured root/node
  j :tpl-rm [--force] <tpl-name>                 remove template (--force if still referenced)
  j :edit                                        open config in $EDITOR (creates default config if missing)
  j :check                                       validate paths exist
  j :config-path                                 print config file path
  j :install <powershell|cmd>                    install shim
  j :uninstall <powershell|cmd>                  remove shim
  j :init <powershell|cmd>                       print shim script to stdout
  j :help | --help | -h                          show this help with roots summary
  j :version | --version                         show version

Exit codes: 0 = success, 1 = internal error, 2 = not found, 3 = config error, 4 = install error
"#;

pub fn help(cfg_path: &Path) -> Result<String, JError> {
    let mut out = String::from(HELP);
    if let Some(summary) = roots_summary(cfg_path) {
        out.push('\n');
        out.push_str(&summary);
    }
    Ok(out)
}

pub fn version() -> Result<String, JError> {
    Ok(format!("j {}\n", env!("CARGO_PKG_VERSION")))
}

/// Returns a formatted roots list, or None if the config cannot be loaded/validated.
/// Failures are silent — help should never fail because of a bad/missing config.
fn roots_summary(cfg_path: &Path) -> Option<String> {
    let src = std::fs::read_to_string(cfg_path).ok()?;
    let cfg = load_from_str_validated(&src, &cfg_path.display().to_string()).ok()?;
    if cfg.roots.is_empty() {
        return Some(format!(
            "Roots (none configured; edit {})\n",
            cfg_path.display()
        ));
    }
    let mut s = String::from("Roots:\n");
    for (name, node) in &cfg.roots {
        s.push_str(&format!("  {:<12} {}\n", name, node.path));
    }
    Some(s)
}
