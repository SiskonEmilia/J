use crate::merge::effective_children;
use crate::model::Config;
use crate::resolve::resolve;
use std::path::{Path, PathBuf};

const COMMON_SUBCMDS: &[&str] = &[
    ":list",
    ":add",
    ":rm",
    ":alias",
    ":tpl-dump",
    ":tpl-apply",
    ":tpl-rm",
    ":edit",
    ":check",
    ":config-path",
    ":install",
    ":uninstall",
    ":init",
    ":help",
    ":version",
];

/// 返回候选列表，每项为 (符号, 展示路径)。
/// 展示路径：root 用绝对路径；符号用相对路径（/ 分隔）；别名用命令；子命令为空。
pub fn complete_rich(line: &str, cursor: usize, cfg: &Config) -> Vec<(String, String)> {
    let trimmed = &line[..cursor.min(line.len())];
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let ends_space = trimmed.ends_with(|c: char| c.is_whitespace());

    let after_j: Vec<&str> = if let Some(first) = tokens.first() {
        if *first == "j" {
            tokens[1..].to_vec()
        } else {
            tokens.clone()
        }
    } else {
        vec![]
    };

    // position == 当前正在编辑的 token 的索引（从 argv 首 token 开始，跳过 program name "j"）
    // 若 ends_space，正在编辑一个新 token
    let (cur_idx, partial): (usize, &str) = if ends_space {
        (after_j.len(), "")
    } else if after_j.is_empty() {
        (0, "")
    } else {
        (after_j.len() - 1, after_j.last().copied().unwrap_or(""))
    };

    let is_first = cur_idx == 0;

    // 子命令：首 token 以 ':' 开头
    if is_first && partial.starts_with(':') {
        return COMMON_SUBCMDS
            .iter()
            .filter(|c| c.starts_with(partial))
            .map(|s| (s.to_string(), String::new()))
            .collect();
    }
    if is_first && partial.starts_with('-') {
        return cfg
            .commands
            .iter()
            .map(|(k, v)| (format!("-{}", k), v.clone()))
            .filter(|(s, _)| s.starts_with(partial))
            .collect();
    }
    if let Some(":add") = after_j.first().copied() {
        return complete_add_rich(&after_j, cur_idx, partial, ends_space, cfg);
    }
    if let Some(":tpl-rm") = after_j.first().copied() {
        if cur_idx == 1 {
            return cfg
                .templates
                .keys()
                .filter(|k| k.starts_with(partial))
                .map(|k| (k.clone(), String::new()))
                .collect();
        }
    }
    if let Some(_cmd @ (":install" | ":uninstall" | ":init")) = after_j.first().copied() {
        if cur_idx == 1 {
            return ["powershell", "cmd"]
                .iter()
                .filter(|s| s.starts_with(partial))
                .map(|s| (s.to_string(), String::new()))
                .collect();
        }
    }

    // 首 token：所有 root + 常用子命令
    if is_first {
        let roots: Vec<(String, String)> = cfg
            .roots
            .iter()
            .filter(|(k, _)| k.starts_with(partial))
            .map(|(k, v)| (k.clone(), v.path.clone()))
            .collect();

        // 精确匹配某个根 → 展开其子符号
        if roots.len() == 1 && roots[0].0 == partial {
            let root = cfg.roots.get(partial).unwrap();
            let children = effective_children(root, cfg);
            if !children.is_empty() {
                return children
                    .iter()
                    .map(|(k, v)| (k.clone(), v.path.replace('\\', "/")))
                    .collect();
            }
        }

        let mut out = roots;
        for c in COMMON_SUBCMDS {
            if c.starts_with(partial) {
                out.push(((*c).to_string(), String::new()));
            }
        }
        return out;
    }

    // 是否已经进入 -alias 段？
    let alias_idx = after_j.iter().position(|t| t.starts_with('-'));
    if let Some(i) = alias_idx {
        if cur_idx > i {
            return Vec::new();
        }
        if cur_idx == i {
            return cfg
                .commands
                .iter()
                .map(|(k, v)| (format!("-{}", k), v.clone()))
                .filter(|(s, _)| s.starts_with(partial))
                .collect();
        }
    }

    // 否则：在 root 下按路径逐级到 cur_idx-1，补 cur_idx 位置
    let root_name = after_j.first().copied().unwrap_or("");
    let Some(root) = cfg.roots.get(root_name) else {
        return Vec::new();
    };

    let mut view = effective_children(root, cfg);
    for sym in after_j.iter().take(cur_idx).skip(1) {
        let Some(next) = view.get(*sym) else {
            return Vec::new();
        };
        view = next.children.clone();
    }
    view.iter()
        .filter(|(k, _)| k.starts_with(partial))
        .map(|(k, v)| (k.clone(), v.path.replace('\\', "/")))
        .collect()
}

/// `complete_rich` 的瘦包装：丢弃展示路径，仅返回候选符号列表，供
/// `:complete` 子命令（和非交互 shell 原生补全）使用。
pub fn complete(line: &str, cursor: usize, cfg: &Config) -> Vec<String> {
    complete_rich(line, cursor, cfg)
        .into_iter()
        .map(|(s, _)| s)
        .collect()
}

fn complete_add_rich(
    after_j: &[&str],
    cur_idx: usize,
    partial: &str,
    ends_space: bool,
    cfg: &Config,
) -> Vec<(String, String)> {
    if cur_idx == 1 {
        return cfg
            .roots
            .keys()
            .filter(|k| k.starts_with(partial))
            .map(|k| (k.clone(), String::new()))
            .collect();
    }

    let root_name = after_j.get(1).copied().unwrap_or("");
    let Some(root) = cfg.roots.get(root_name) else {
        return Vec::new();
    };

    let add_args = &after_j[2..];
    let fixed = if ends_space || add_args.is_empty() {
        add_args
    } else {
        &add_args[..add_args.len() - 1]
    };

    if fixed.is_empty() && !looks_like_path_fragment(partial) {
        return effective_children(root, cfg)
            .iter()
            .filter(|(k, _)| k.starts_with(partial))
            .map(|(k, v)| (k.clone(), v.path.replace('\\', "/")))
            .collect();
    }

    let Some(view) = resolved_children_at(root_name, fixed, cfg) else {
        return Vec::new();
    };

    if !looks_like_path_fragment(partial) {
        let symbol_matches: Vec<(String, String)> = view
            .iter()
            .filter(|(k, _)| k.starts_with(partial))
            .map(|(k, v)| (k.clone(), v.path.replace('\\', "/")))
            .collect();
        if !symbol_matches.is_empty() {
            return symbol_matches;
        }
    }

    let Some(base_dir) = resolved_abs_dir(root_name, fixed, cfg) else {
        return Vec::new();
    };
    complete_dir_fragment(&base_dir, partial)
        .into_iter()
        .map(|s| (s, String::new()))
        .collect()
}

fn resolved_children_at(
    root_name: &str,
    syms: &[&str],
    cfg: &Config,
) -> Option<std::collections::BTreeMap<String, crate::merge::ResolvedChild>> {
    let root = cfg.roots.get(root_name)?;
    let mut view = effective_children(root, cfg);
    for sym in syms {
        let next = view.get(*sym)?.clone();
        view = next.children;
    }
    Some(view)
}

fn resolved_abs_dir(root_name: &str, syms: &[&str], cfg: &Config) -> Option<PathBuf> {
    let mut positional: Vec<&str> = Vec::with_capacity(1 + syms.len());
    positional.push(root_name);
    positional.extend_from_slice(syms);
    let resolved = resolve(&positional, None, &[], cfg).ok()?;
    Some(PathBuf::from(resolved.abs_path))
}

fn complete_dir_fragment(base_dir: &Path, partial: &str) -> Vec<String> {
    let partial_path = Path::new(partial);
    let (scan_dir, prefix, name_prefix) = if partial.is_empty() {
        (base_dir.to_path_buf(), String::new(), String::new())
    } else if partial_path.is_absolute() {
        split_absolute_completion(partial_path)
    } else {
        split_relative_completion(base_dir, partial_path)
    };

    let read_dir = match std::fs::read_dir(&scan_dir) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let name_prefix_lower = name_prefix.to_ascii_lowercase();
    let mut out = Vec::new();
    for entry in read_dir.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if !ft.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.to_ascii_lowercase().starts_with(&name_prefix_lower) {
            continue;
        }
        out.push(format!("{}{}\\", prefix, name));
    }
    out.sort();
    out
}

fn split_relative_completion(base_dir: &Path, partial: &Path) -> (PathBuf, String, String) {
    let raw = partial.to_string_lossy().replace('/', "\\");
    if raw.ends_with('\\') {
        return (base_dir.join(partial), raw, String::new());
    }
    let parent = partial.parent().filter(|p| !p.as_os_str().is_empty());
    let scan_dir = match parent {
        Some(p) => base_dir.join(p),
        None => base_dir.to_path_buf(),
    };
    let prefix = match parent {
        Some(p) => {
            let mut s = p.to_string_lossy().replace('/', "\\");
            if !s.ends_with('\\') {
                s.push('\\');
            }
            s
        }
        None => String::new(),
    };
    let name_prefix = partial
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    (scan_dir, prefix, name_prefix)
}

fn split_absolute_completion(partial: &Path) -> (PathBuf, String, String) {
    let raw = partial.to_string_lossy().replace('/', "\\");
    if raw.ends_with('\\') {
        return (PathBuf::from(&raw), raw, String::new());
    }
    let parent = partial.parent().unwrap_or_else(|| Path::new(&raw));
    let mut prefix = parent.to_string_lossy().replace('/', "\\");
    if !prefix.is_empty() && !prefix.ends_with('\\') {
        prefix.push('\\');
    }
    let name_prefix = partial
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    (PathBuf::from(parent), prefix, name_prefix)
}

fn looks_like_path_fragment(s: &str) -> bool {
    s.contains('\\') || s.contains('/') || s.contains('.') || s.contains(':')
}
