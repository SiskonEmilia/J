use crate::complete::complete_rich as compute;
use crate::config::load_from_str_validated;
use crate::error::JError;
use std::path::Path;

pub fn run(cfg_path: &Path, args: &[String]) -> Result<String, JError> {
    // args: <shell> <cursor> <line>
    if args.len() < 3 {
        return Ok(String::new());
    }
    let cursor: usize = args[1].parse().unwrap_or(0);
    let line = &args[2];
    let src = std::fs::read_to_string(cfg_path).unwrap_or_else(|_| "{\"roots\":{}}".into());
    let cfg = match load_from_str_validated(&src, &cfg_path.display().to_string()) {
        Ok(c) => c,
        Err(_) => return Ok(String::new()),
    };
    let candidates = compute(line, cursor, &cfg);
    let out: String = candidates.iter()
        .map(|(sym, path)| format!("{}\t{}\n", sanitize(sym), sanitize(path)))
        .collect();
    Ok(out)
}

fn sanitize(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}
