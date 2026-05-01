use crate::error::JError;
use crate::merge::{effective_children, ResolvedChild};
use crate::model::{Config, Node};
use std::fmt::Write;

pub fn run(cfg: &Config, args: &[String]) -> Result<String, JError> {
    let mut out = String::new();
    if args.is_empty() {
        for (name, root) in &cfg.roots {
            writeln!(&mut out, "{} -> {}", name, root.path).unwrap();
            print_children_literal(root, cfg, 1, &mut out);
        }
        if !cfg.commands.is_empty() {
            writeln!(&mut out, "\nCommands:").unwrap();
            for (k, v) in &cfg.commands {
                writeln!(&mut out, "  -{} = {}", k, v).unwrap();
            }
        }
        if !cfg.templates.is_empty() {
            writeln!(&mut out, "\nTemplates:").unwrap();
            for name in cfg.templates.keys() {
                writeln!(&mut out, "  {}", name).unwrap();
            }
        }
    } else {
        let root_name = &args[0];
        let root = cfg.roots.get(root_name).ok_or_else(|| JError::UnknownRoot {
            name: root_name.clone(),
            available: cfg.roots.keys().cloned().collect(),
        })?;
        writeln!(&mut out, "{} -> {}", root_name, root.path).unwrap();
        let rest: Vec<&str> = args[1..].iter().map(String::as_str).collect();
        render_downpath(root, cfg, &rest, 1, &mut out)?;
    }
    Ok(out)
}

fn print_children_literal(n: &Node, cfg: &Config, indent: usize, out: &mut String) {
    let view = effective_children(n, cfg);
    for (k, v) in &view {
        let prefix = "  ".repeat(indent);
        let src = if v.source == "self" { String::new() } else { format!("  ({})", v.source) };
        writeln!(out, "{}{}  -> {}{}", prefix, k, v.path, src).unwrap();
        print_resolved(v, indent + 1, out);
    }
}

fn print_resolved(rc: &ResolvedChild, indent: usize, out: &mut String) {
    for (k, v) in &rc.children {
        let prefix = "  ".repeat(indent);
        let src = if v.source == "self" { String::new() } else { format!("  ({})", v.source) };
        writeln!(out, "{}{}  -> {}{}", prefix, k, v.path, src).unwrap();
        print_resolved(v, indent + 1, out);
    }
}

fn render_downpath(root: &Node, cfg: &Config, rest: &[&str], indent: usize, out: &mut String)
    -> Result<(), JError>
{
    if rest.is_empty() {
        print_children_literal(root, cfg, indent, out);
        return Ok(());
    }
    let view = effective_children(root, cfg);
    let next = view.get(rest[0]).ok_or_else(|| JError::UnknownSymbol {
        path: vec![],
        sym: rest[0].to_string(),
        available: view.iter().map(|(k, v)| (k.clone(), Some(v.source.clone()))).collect(),
    })?;
    let prefix = "  ".repeat(indent);
    writeln!(out, "{}{}  -> {}", prefix, rest[0], next.path).unwrap();
    if rest.len() > 1 {
        let mut cur = next.clone();
        for (i, s) in rest[1..].iter().enumerate() {
            let child = cur.children.get(*s).ok_or_else(|| JError::UnknownSymbol {
                path: rest[..=i].iter().map(|s| s.to_string()).collect(),
                sym: s.to_string(),
                available: cur.children.iter().map(|(k, v)| (k.clone(), Some(v.source.clone()))).collect(),
            })?.clone();
            writeln!(out, "{}{}  -> {}", "  ".repeat(indent + 1 + i), s, child.path).unwrap();
            cur = child;
        }
        print_resolved(&cur, indent + rest.len(), out);
    } else {
        print_resolved(next, indent + 1, out);
    }
    Ok(())
}
