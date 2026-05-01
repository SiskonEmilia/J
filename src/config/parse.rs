use crate::error::JError;
use crate::model::{Config, Node, Template, TemplateNode};
use jsonc_parser::{parse_to_serde_value, ParseOptions};
use serde_json::Value;
use std::collections::BTreeMap;

pub fn load_from_str(src: &str, path: &str) -> Result<Config, JError> {
    let value = parse_to_serde_value(src, &ParseOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        allow_loose_object_property_names: false,
    })
    .map_err(|e| JError::ConfigError {
        path: path.to_string(),
        line: e.line_display(),
        col: e.column_display(),
        msg: e.kind().to_string(),
    })?
    .ok_or_else(|| JError::ConfigInvalid {
        msg: format!("{}: top-level must be an object", path),
    })?;

    let obj = match value {
        Value::Object(m) => m,
        _ => return Err(JError::ConfigInvalid {
            msg: format!("{}: top-level must be an object", path),
        }),
    };

    let commands = match obj.get("commands") {
        Some(Value::Object(m)) => {
            let mut out = BTreeMap::new();
            for (k, v) in m {
                let s = v.as_str().ok_or_else(|| JError::ConfigInvalid {
                    msg: format!("commands.{}: value must be a string", k),
                })?;
                out.insert(k.clone(), s.to_string());
            }
            out
        }
        Some(_) => return Err(JError::ConfigInvalid { msg: "commands must be an object".into() }),
        None => BTreeMap::new(),
    };

    let templates = match obj.get("templates") {
        Some(Value::Object(m)) => {
            let mut out = BTreeMap::new();
            for (k, v) in m {
                out.insert(k.clone(), parse_template(v, &format!("templates.{}", k))?);
            }
            out
        }
        Some(_) => return Err(JError::ConfigInvalid { msg: "templates must be an object".into() }),
        None => BTreeMap::new(),
    };

    let roots = match obj.get("roots") {
        Some(Value::Object(m)) => {
            let mut out = BTreeMap::new();
            for (k, v) in m {
                out.insert(k.clone(), parse_node(v, &format!("roots.{}", k))?);
            }
            out
        }
        _ => return Err(JError::ConfigInvalid { msg: "top-level key 'roots' is required and must be an object".into() }),
    };

    Ok(Config { commands, templates, roots })
}

fn parse_node(v: &Value, ctx: &str) -> Result<Node, JError> {
    let m = v.as_object().ok_or_else(|| JError::ConfigInvalid {
        msg: format!("{}: must be an object", ctx),
    })?;
    let path = m.get("path").and_then(|x| x.as_str())
        .ok_or_else(|| JError::ConfigInvalid { msg: format!("{}.path: required string", ctx) })?
        .to_string();
    let templates = match m.get("templates") {
        Some(Value::Array(arr)) => arr.iter().map(|x| x.as_str()
            .ok_or_else(|| JError::ConfigInvalid { msg: format!("{}.templates: each entry must be a string", ctx) })
            .map(String::from))
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(JError::ConfigInvalid { msg: format!("{}.templates: must be an array", ctx) }),
        None => Vec::new(),
    };
    let children = parse_children_node(m.get("children"), ctx)?;
    Ok(Node { path, templates, children })
}

fn parse_children_node(v: Option<&Value>, ctx: &str) -> Result<BTreeMap<String, Node>, JError> {
    match v {
        None => Ok(BTreeMap::new()),
        Some(Value::Object(m)) => {
            let mut out = BTreeMap::new();
            for (k, v) in m {
                out.insert(k.clone(), parse_node(v, &format!("{}.children.{}", ctx, k))?);
            }
            Ok(out)
        }
        Some(_) => Err(JError::ConfigInvalid { msg: format!("{}.children: must be an object", ctx) }),
    }
}

fn parse_template(v: &Value, ctx: &str) -> Result<Template, JError> {
    let m = v.as_object().ok_or_else(|| JError::ConfigInvalid {
        msg: format!("{}: must be an object", ctx),
    })?;
    if m.contains_key("templates") {
        return Err(JError::ConfigInvalid {
            msg: format!("{}.templates: templates cannot recursively mix in other templates", ctx),
        });
    }
    let children = parse_children_template(m.get("children"), ctx)?;
    Ok(Template { children })
}

fn parse_children_template(v: Option<&Value>, ctx: &str)
    -> Result<BTreeMap<String, TemplateNode>, JError>
{
    match v {
        None => Ok(BTreeMap::new()),
        Some(Value::Object(m)) => {
            let mut out = BTreeMap::new();
            for (k, v) in m {
                let sub = v.as_object().ok_or_else(|| JError::ConfigInvalid {
                    msg: format!("{}.children.{}: must be an object", ctx, k),
                })?;
                if sub.contains_key("templates") {
                    return Err(JError::ConfigInvalid {
                        msg: format!("{}.children.{}.templates: templates cannot be nested mixin", ctx, k),
                    });
                }
                let path = sub.get("path").and_then(|x| x.as_str())
                    .ok_or_else(|| JError::ConfigInvalid { msg: format!("{}.children.{}.path: required string", ctx, k) })?
                    .to_string();
                let kids = parse_children_template(sub.get("children"), &format!("{}.children.{}", ctx, k))?;
                out.insert(k.clone(), TemplateNode { path, children: kids });
            }
            Ok(out)
        }
        Some(_) => Err(JError::ConfigInvalid { msg: format!("{}.children: must be an object", ctx) }),
    }
}
