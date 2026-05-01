use crate::error::JError;

pub const BEGIN: &str = "# region j-shim (do not edit between markers)";
pub const END:   &str = "# endregion j-shim";

/// Upsert a block wrapped with BEGIN/END markers.
/// If markers already exist the whole block is replaced; otherwise appended.
pub fn upsert(content: &str, block_body: &str) -> String {
    let marker_begin = BEGIN;
    let marker_end   = END;
    if let (Some(b), Some(e)) = (content.find(marker_begin), content.find(marker_end)) {
        if e > b {
            let end_of_region = e + marker_end.len();
            let mut out = String::new();
            out.push_str(&content[..b]);
            out.push_str(marker_begin);
            out.push('\n');
            out.push_str(block_body.trim_end_matches('\n'));
            out.push('\n');
            out.push_str(marker_end);
            out.push_str(&content[end_of_region..]);
            return out;
        }
    }
    let mut out = content.to_string();
    if !out.is_empty() && !out.ends_with('\n') { out.push('\n'); }
    if !out.is_empty() { out.push('\n'); } // blank line separator
    out.push_str(marker_begin);
    out.push('\n');
    out.push_str(block_body.trim_end_matches('\n'));
    out.push('\n');
    out.push_str(marker_end);
    out.push('\n');
    out
}

pub fn remove(content: &str) -> Result<String, JError> {
    let b = content.find(BEGIN);
    let e = content.find(END);
    match (b, e) {
        (Some(b), Some(e)) if e > b => {
            let end_of = e + END.len();
            let mut out = String::new();
            out.push_str(&content[..b]);
            // skip the newline right after region if present
            let after = &content[end_of..];
            let after_trim = after.strip_prefix('\n').unwrap_or(after);
            out.push_str(after_trim);
            Ok(collapse_blank_runs(&out))
        }
        _ => Ok(content.to_string()),
    }
}

fn collapse_blank_runs(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_blank = false;
    for line in s.lines() {
        if line.trim().is_empty() {
            if !last_blank { out.push('\n'); }
            last_blank = true;
        } else {
            out.push_str(line);
            out.push('\n');
            last_blank = false;
        }
    }
    if !s.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}
