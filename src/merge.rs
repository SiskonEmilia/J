use crate::model::{Config, Node, TemplateNode};
use std::collections::BTreeMap;

/// 合并后子节点的视图。来源标记便于错误信息与补全展示。
#[derive(Debug, Clone)]
pub struct ResolvedChild {
    pub path: String,
    pub children: BTreeMap<String, ResolvedChild>,
    pub source: String, // "self" | template name
}

pub fn effective_children(n: &Node, c: &Config) -> BTreeMap<String, ResolvedChild> {
    let mut out: BTreeMap<String, ResolvedChild> = BTreeMap::new();

    for tpl_name in &n.templates {
        if let Some(tpl) = c.templates.get(tpl_name) {
            for (k, v) in &tpl.children {
                let incoming = from_template(v, tpl_name);
                out.entry(k.clone())
                    .and_modify(|existing| *existing = merge_pair(existing, &incoming))
                    .or_insert(incoming);
            }
        }
    }

    for (k, v) in &n.children {
        let incoming = from_self(v);
        out.entry(k.clone())
            .and_modify(|existing| *existing = merge_pair(existing, &incoming))
            .or_insert(incoming);
    }

    out
}

fn from_template(t: &TemplateNode, src: &str) -> ResolvedChild {
    ResolvedChild {
        path: t.path.clone(),
        children: t.children.iter().map(|(k, v)| (k.clone(), from_template(v, src))).collect(),
        source: src.to_string(),
    }
}

fn from_self(n: &Node) -> ResolvedChild {
    // Node 自身也可能挂模板，但这里只看"直接子节点的字面定义"；
    // 递归解析要用 effective_children_of_subtree（Node 上的模板只在 jump 下钻时再解析）
    ResolvedChild {
        path: n.path.clone(),
        children: n.children.iter().map(|(k, v)| (k.clone(), from_self(v))).collect(),
        source: "self".to_string(),
    }
}

fn merge_pair(old: &ResolvedChild, new: &ResolvedChild) -> ResolvedChild {
    let mut merged_children = old.children.clone();
    for (k, v) in &new.children {
        merged_children.entry(k.clone())
            .and_modify(|e| *e = merge_pair(e, v))
            .or_insert_with(|| v.clone());
    }
    ResolvedChild {
        path: new.path.clone(),        // 后者胜
        children: merged_children,
        source: new.source.clone(),    // 来源跟随后者
    }
}
