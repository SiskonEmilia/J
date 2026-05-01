use j::config::load_from_str_validated;
use j::merge::effective_children;

const CFG: &str = r#"
{
  "templates": {
    "tpl1": { "children": {
      "a": { "path": "A" },
      "common": { "path": "C1" }
    }},
    "tpl2": { "children": {
      "b": { "path": "B" },
      "common": { "path": "C2" }
    }}
  },
  "roots": {
    "r": {
      "path": "C:\\r",
      "templates": ["tpl1", "tpl2"],
      "children": {
        "a": { "path": "A_OVERRIDE" },
        "own": { "path": "OWN" }
      }
    }
  }
}
"#;

#[test]
fn root_own_children_win() {
    let c = load_from_str_validated(CFG, "x").unwrap();
    let root = c.roots.get("r").unwrap();
    let view = effective_children(root, &c);
    let a = view.get("a").unwrap();
    assert_eq!(a.path, "A_OVERRIDE");
    assert_eq!(a.source, "self");
}

#[test]
fn later_template_overrides_earlier() {
    let c = load_from_str_validated(CFG, "x").unwrap();
    let view = effective_children(c.roots.get("r").unwrap(), &c);
    let common = view.get("common").unwrap();
    assert_eq!(common.path, "C2");
    assert_eq!(common.source, "tpl2");
}

#[test]
fn earlier_template_unique_child_preserved() {
    let c = load_from_str_validated(CFG, "x").unwrap();
    let view = effective_children(c.roots.get("r").unwrap(), &c);
    let b = view.get("b").unwrap();
    assert_eq!(b.path, "B");
    assert_eq!(b.source, "tpl2");
    let b_from_self = view.get("own").unwrap();
    assert_eq!(b_from_self.source, "self");
}

#[test]
fn deep_merge_recursive() {
    let src = r#"{
      "templates": {"t": {"children": {
        "src": {"path":"Source","children":{"pri":{"path":"Private"}}}
      }}},
      "roots": {"r": {"path":"C:\\r","templates":["t"],"children":{
        "src": {"path":"SRC","children":{"pub":{"path":"Public"}}}
      }}}
    }"#;
    let c = load_from_str_validated(src, "x").unwrap();
    let view = effective_children(c.roots.get("r").unwrap(), &c);
    let src_child = view.get("src").unwrap();
    // path 来自 self（self 为最高优先级）
    assert_eq!(src_child.path, "SRC");
    // children 深度合并：模板贡献 pri，self 贡献 pub
    assert!(src_child.children.contains_key("pri"));
    assert!(src_child.children.contains_key("pub"));
}
