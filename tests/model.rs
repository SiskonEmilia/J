use j::model::{Config, Node, Template, TemplateNode};
use std::collections::BTreeMap;

#[test]
fn config_roundtrip_via_struct_literals() {
    let c = Config {
        commands: BTreeMap::from([("c".into(), "code".into())]),
        templates: BTreeMap::from([("u".into(), Template {
            children: BTreeMap::from([("d".into(), TemplateNode {
                path: "Data".into(),
                children: BTreeMap::new(),
            })]),
        })]),
        roots: BTreeMap::from([("d3".into(), Node {
            path: "C:\\d3".into(),
            templates: vec!["u".into()],
            children: BTreeMap::new(),
        })]),
    };
    assert_eq!(c.roots.get("d3").unwrap().templates, vec!["u".to_string()]);
}
