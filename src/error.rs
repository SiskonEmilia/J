use std::fmt;

#[derive(Debug)]
pub enum JError {
    UnknownRoot      { name: String, available: Vec<String> },
    UnknownSymbol    { path: Vec<String>, sym: String, available: Vec<(String, Option<String>)> },
    UnknownAlias     { name: String, available: Vec<String> },
    ConfigError      { path: String, line: usize, col: usize, msg: String },
    ConfigInvalid    { msg: String },
    InstallError     { msg: String },
    Internal         { msg: String },
}

pub fn exit_code(e: &JError) -> i32 {
    match e {
        JError::UnknownRoot { .. } | JError::UnknownSymbol { .. } | JError::UnknownAlias { .. } => 2,
        JError::ConfigError { .. } | JError::ConfigInvalid { .. }                                => 3,
        JError::InstallError { .. }                                                              => 4,
        JError::Internal { .. }                                                                  => 1,
    }
}

impl fmt::Display for JError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JError::UnknownRoot { name, available } => {
                writeln!(f, "j: unknown-root: '{}' is not a configured root.", name)?;
                write!(f, "  available: {}", available.join(", "))
            }
            JError::UnknownSymbol { path, sym, available } => {
                writeln!(f, "j: unknown-symbol: '{}' not found under {}", sym, path.join("."))?;
                let list: Vec<String> = available.iter().map(|(n, src)| match src {
                    Some(s) => format!("{} (from {})", n, s),
                    None    => n.clone(),
                }).collect();
                write!(f, "  available: {}", list.join(", "))
            }
            JError::UnknownAlias { name, available } => {
                writeln!(f, "j: unknown-alias: '-{}'", name)?;
                write!(f, "  available: {}", available.iter().map(|s| format!("-{}", s)).collect::<Vec<_>>().join(", "))
            }
            JError::ConfigError { path, line, col, msg } =>
                write!(f, "j: config-error: {}:{}:{}: {}", path, line, col, msg),
            JError::ConfigInvalid { msg } =>
                write!(f, "j: config-invalid: {}", msg),
            JError::InstallError { msg } =>
                write!(f, "j: install-error: {}", msg),
            JError::Internal { msg } =>
                write!(f, "j: internal-error: {}", msg),
        }
    }
}
