pub mod list;
pub mod config_path;
pub mod help;
pub mod check;
pub mod edit;
pub mod add;
pub mod rm;
pub mod alias;
pub mod tpl;
pub mod install;
pub mod complete;
pub mod complete_rich;

use crate::config::load_from_str_validated;
use crate::error::JError;
use crate::model::Config;
use std::path::Path;

pub fn dispatch(name: &str, args: &[String], cfg_path: &Path) -> Result<String, JError> {
    // subcommands that don't need a loaded config
    match name {
        "config-path" => return config_path::run(cfg_path),
        "help"        => return help::help(cfg_path),
        "version"     => return help::version(),
        "edit"        => return edit::run(cfg_path),
        "add"         => return add::run(cfg_path, args),
        "rm"          => return rm::run(cfg_path, args),
        "alias"       => return alias::run(cfg_path, args),
        "tpl-dump"    => return tpl::dump(cfg_path, args),
        "tpl-apply"   => return tpl::apply(cfg_path, args),
        "tpl-rm"      => return tpl::remove(cfg_path, args),
        "install"     => return install::install(cfg_path, args),
        "uninstall"   => return install::uninstall(cfg_path, args),
        "init"        => return install::init(args),
        "complete"    => return complete::run(cfg_path, args),
        "complete-rich" => return complete_rich::run(cfg_path, args),
        _ => {}
    }
    let cfg = load(cfg_path)?;
    match name {
        "list"  => list::run(&cfg, args),
        "check" => check::run(&cfg),
        _ => Err(JError::Internal { msg: format!("unknown subcommand ':{}'", name) }),
    }
}

fn load(path: &Path) -> Result<Config, JError> {
    let src = std::fs::read_to_string(path).map_err(|e| JError::ConfigError {
        path: path.display().to_string(), line: 0, col: 0,
        msg: format!("cannot read: {}", e),
    })?;
    load_from_str_validated(&src, &path.display().to_string())
}
