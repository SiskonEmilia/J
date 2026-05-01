use j::complete::complete;
use j::complete::complete_rich;
use j::config::load_from_str_validated;
use tempfile::tempdir;

const FIXTURE: &str = include_str!("fixtures/config.jsonc");

fn cfg() -> j::model::Config {
    load_from_str_validated(FIXTURE, "x").unwrap()
}

fn suggest(line: &str) -> Vec<String> {
    let cur = line.len();
    complete(line, cur, &cfg())
}

#[test]
fn first_token_partial_root() {
    let v = suggest("j d");
    assert!(v.iter().any(|s| s == "d3"));
    // 同时包含常用子命令（首 token 可补 :list 等）
}

#[test]
fn second_token_lists_children() {
    let v = suggest("j d3 ");
    assert!(v.iter().any(|s| s == "d"));
    assert!(v.iter().any(|s| s == "sd"));
    assert!(v.iter().any(|s| s == "notes"));
}

#[test]
fn dash_lists_aliases() {
    let v = suggest("j d3 d -");
    assert!(v.iter().any(|s| s == "-c"));
    assert!(v.iter().any(|s| s == "-cc"));
}

#[test]
fn first_token_dash_lists_aliases() {
    let v = suggest("j -");
    assert!(v.iter().any(|s| s == "-c"));
    assert!(v.iter().any(|s| s == "-cc"));
}

#[test]
fn after_alias_no_suggestion() {
    let v = suggest("j d3 d -c --");
    assert!(v.is_empty());
}

#[test]
fn colon_subcmd() {
    let v = suggest("j :");
    assert!(v.iter().any(|s| s == ":list"));
    assert!(v.iter().any(|s| s == ":install"));
}

#[test]
fn colon_tpl_rm_completes_template_names() {
    let v = suggest("j :tpl-rm ");
    assert!(v.iter().any(|s| s == "uProject"));
}

#[test]
fn add_prefers_symbol_completion_before_path_completion() {
    let v = suggest("j :add d3 s");
    assert!(v.iter().any(|s| s == "sd"));
    assert!(v.iter().any(|s| s == "src"));
}

#[test]
fn add_completes_relative_directories_under_resolved_symbol_path() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("root");
    std::fs::create_dir_all(root.join("docs\\notes\\drafts")).unwrap();
    std::fs::create_dir_all(root.join("docs\\notes\\done")).unwrap();
    std::fs::write(root.join("docs\\notes\\todo.txt"), "x").unwrap();

    let cfg_src = format!(
        r#"{{
  "roots": {{
    "r": {{
      "path": "{}",
      "children": {{
        "notes": {{ "path": "docs\\notes" }}
      }}
    }}
  }}
}}"#,
        root.display().to_string().replace('\\', "\\\\")
    );
    let cfg = load_from_str_validated(&cfg_src, "x").unwrap();

    let v = complete("j :add r notes d", "j :add r notes d".len(), &cfg);
    assert!(v.iter().any(|s| s == "done\\"));
    assert!(v.iter().any(|s| s == "drafts\\"));
    assert!(!v.iter().any(|s| s.contains("todo.txt")));
}

#[test]
fn add_completes_nested_relative_directories() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("root");
    std::fs::create_dir_all(root.join("docs\\notes\\drafts\\alpha")).unwrap();
    std::fs::create_dir_all(root.join("docs\\notes\\drafts\\beta")).unwrap();

    let cfg_src = format!(
        r#"{{
  "roots": {{
    "r": {{
      "path": "{}",
      "children": {{
        "notes": {{ "path": "docs\\notes" }}
      }}
    }}
  }}
}}"#,
        root.display().to_string().replace('\\', "\\\\")
    );
    let cfg = load_from_str_validated(&cfg_src, "x").unwrap();

    let v = complete(
        "j :add r notes drafts\\a",
        "j :add r notes drafts\\a".len(),
        &cfg,
    );
    assert_eq!(v, vec!["drafts\\alpha\\".to_string()]);
}

#[test]
fn add_returns_no_completion_when_symbol_path_cannot_be_resolved() {
    let v = suggest("j :add d3 missing x");
    assert!(v.is_empty());
}

#[test]
fn rich_exact_root_match_expands_children() {
    // 精确匹配有子符号的根 → 返回子符号列表，不返回根本身
    let v = complete_rich("j d3", 4, &cfg());
    assert!(v.iter().any(|(s, _)| s == "d"));
    assert!(v.iter().any(|(s, _)| s == "sd"));
    assert!(v.iter().any(|(s, _)| s == "src"));
    assert!(v.iter().any(|(s, _)| s == "notes"));
    assert!(!v.iter().any(|(s, _)| s == "d3"));
}

#[test]
fn rich_exact_root_match_returns_root_when_no_children() {
    let src = r#"{"roots":{"leaf":{"path":"C:\\leaf"}},"templates":{},"commands":{}}"#;
    let cfg = load_from_str_validated(src, "x").unwrap();
    let v = complete_rich("j leaf", 6, &cfg);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].0, "leaf");
    assert_eq!(v[0].1, "C:\\leaf");
}

#[test]
fn rich_first_token_subcmd_shows_empty_path() {
    let v = complete_rich("j :", 3, &cfg());
    let entry = v
        .iter()
        .find(|(s, _)| s == ":list")
        .expect(":list not found");
    assert_eq!(entry.1, "");
}

#[test]
fn rich_symbol_shows_relative_path_with_forward_slash() {
    // d3 の子符号 notes 来自 self，path = "docs\\notes"
    let v = complete_rich("j d3 ", 5, &cfg());
    let entry = v
        .iter()
        .find(|(s, _)| s == "notes")
        .expect("notes not found");
    assert_eq!(entry.1, "docs/notes");
}

#[test]
fn rich_template_symbol_shows_path() {
    // d 来自 uProject 模板，path = "Data"
    let v = complete_rich("j d3 ", 5, &cfg());
    let entry = v.iter().find(|(s, _)| s == "d").expect("d not found");
    assert_eq!(entry.1, "Data");
}

#[test]
fn rich_alias_shows_command_string() {
    let v = complete_rich("j d3 d -", 8, &cfg());
    let entry = v.iter().find(|(s, _)| s == "-c").expect("-c not found");
    assert_eq!(entry.1, "code");
}

#[test]
fn rich_symbols_count_matches_plain_complete() {
    // complete_rich 的符号集合与 complete 相同，顺序一致
    let plain = complete("j d3 ", 5, &cfg());
    let rich: Vec<String> = complete_rich("j d3 ", 5, &cfg())
        .into_iter()
        .map(|(s, _)| s)
        .collect();
    assert_eq!(plain, rich);
}
