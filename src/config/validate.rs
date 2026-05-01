use crate::error::JError;
use crate::model::{Config, Node, TemplateNode};
use std::path::Path;

pub fn validate(c: &Config) -> Result<(), JError> {
    for (name, node) in &c.roots {
        check_symbol(name, "root name")?;
        if !Path::new(&node.path).is_absolute() {
            return Err(JError::ConfigInvalid {
                msg: format!("root '{}' path '{}' is not absolute", name, node.path),
            });
        }
        validate_templates_ref(&node.templates, c, &format!("root '{}'", name))?;
        walk_node(node, c, &format!("roots.{}", name))?;
    }
    for (name, tpl) in &c.templates {
        check_symbol(name, "template name")?;
        for (k, t) in &tpl.children {
            walk_template(k, t, &format!("templates.{}.children", name))?;
        }
    }
    for name in c.commands.keys() {
        check_symbol(name, "alias name")?;
    }
    Ok(())
}

fn walk_node(n: &Node, c: &Config, ctx: &str) -> Result<(), JError> {
    validate_templates_ref(&n.templates, c, ctx)?;
    for (k, child) in &n.children {
        check_symbol(k, &format!("{}.children key", ctx))?;
        walk_node(child, c, &format!("{}.children.{}", ctx, k))?;
    }
    Ok(())
}

fn walk_template(k: &str, n: &TemplateNode, ctx: &str) -> Result<(), JError> {
    check_symbol(k, &format!("{} key", ctx))?;
    for (kk, child) in &n.children {
        walk_template(kk, child, &format!("{}.{}.children", ctx, k))?;
    }
    Ok(())
}

fn validate_templates_ref(refs: &[String], c: &Config, ctx: &str) -> Result<(), JError> {
    for r in refs {
        if !c.templates.contains_key(r) {
            return Err(JError::ConfigInvalid {
                msg: format!("{} references unknown template '{}'", ctx, r),
            });
        }
    }
    Ok(())
}

fn check_symbol(name: &str, kind: &str) -> Result<(), JError> {
    let mut chars = name.chars();
    let first = chars.next().ok_or_else(|| JError::ConfigInvalid {
        msg: format!("{} cannot be empty", kind),
    })?;
    if !(first.is_ascii_alphanumeric() || first == '_') {
        return Err(JError::ConfigInvalid {
            msg: format!("{} '{}' must start with [A-Za-z0-9_]", kind, name),
        });
    }
    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
            return Err(JError::ConfigInvalid {
                msg: format!("{} '{}' contains illegal char '{}'", kind, name, ch),
            });
        }
    }
    Ok(())
}
