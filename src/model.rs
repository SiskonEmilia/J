use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub path: String,
    pub templates: Vec<String>,
    pub children: BTreeMap<String, Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateNode {
    pub path: String,
    pub children: BTreeMap<String, TemplateNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub children: BTreeMap<String, TemplateNode>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Config {
    pub commands: BTreeMap<String, String>,
    pub templates: BTreeMap<String, Template>,
    pub roots: BTreeMap<String, Node>,
}
