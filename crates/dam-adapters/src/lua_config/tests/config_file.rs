use std::time::Duration;

use dam_application::Config;
use dam_domain::Path;

use super::support::{Folder, config};
use crate::lua_config::LuaConfigFile;

const EXAMPLE: &str = r#"
return {
  owner = "Ada",
  done = { interactive = true },
  remote = {
    todoist = { url = "todoist::", stale = "15m", deadline = "60s", path = "work" },
  },
  category = {
    effort = { values = { "light", "admin", "deep" }, exclusive = true },
  },
  filter = {
    today = { query = "due:today | overdue" },
  },
}
"#;

#[test]
fn config_lua_reads_its_owner_remotes_categories_and_filters() {
    let config = config(EXAMPLE).load().unwrap();
    assert_eq!(config.owner.as_deref(), Some("Ada"));
    assert!(config.done_interactive);
    let remote = config.remote("todoist").unwrap();
    assert_eq!(remote.helper, "todoist");
    assert_eq!(remote.stale, Some(Duration::from_secs(900)));
    assert_eq!(remote.deadline.as_ref().unwrap().text, "60s");
    assert_eq!(remote.path, Some(Path::parse("work").unwrap()));
    assert!(remote.credentials.is_empty());
    let effort = config.categories.category_of_name("effort").unwrap();
    assert_eq!(effort.values, ["light", "admin", "deep"]);
    assert!(effort.exclusive);
    assert_eq!(config.filter("today").unwrap().query, "due:today | overdue");
}

#[test]
fn a_missing_config_lua_reads_as_an_empty_config() {
    assert_eq!(Folder::new().load().unwrap(), Config::default());
}

#[test]
fn a_syntax_error_names_the_file_and_line_and_never_says_toml() {
    let why = config("return {\n  owner = = 1,\n}\n").refusal();
    assert!(why.contains("config.lua:2:"), "{why}");
    assert!(!why.contains("TOML"), "{why}");
}

#[test]
fn an_error_names_the_whole_path_however_long_the_folder_name() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir
        .path()
        .join("a-folder-name-longer-than-lua-keeps".repeat(3));
    std::fs::create_dir(&folder).unwrap();
    let path = folder.join("config.lua");
    std::fs::write(&path, "return {\n  owner = = 1,\n}\n").unwrap();
    let why = LuaConfigFile::new(path.clone())
        .load()
        .unwrap_err()
        .to_string();
    assert!(why.contains(&format!("{}:2:", path.display())), "{why}");
}

#[test]
fn a_runtime_error_names_the_file_and_line() {
    let why = config("local x = nil\nreturn { owner = x.name }\n").refusal();
    assert!(why.contains("config.lua:2:"), "{why}");
    assert!(!why.contains("stack traceback"), "{why}");
}

#[test]
fn a_file_that_returns_no_table_is_refused_by_name() {
    let why = config("return 42\n").refusal();
    assert!(why.contains("config.lua"), "{why}");
    assert!(why.contains("must return a table"), "{why}");
}

#[test]
fn an_unknown_key_is_refused_naming_the_key_and_its_table() {
    assert_eq!(
        config("return { colour = 'red' }").refusal(),
        "config: unknown key colour at the top level of config.lua"
    );
    let why = config("return { done = { interactive = true, loud = true } }").refusal();
    assert_eq!(why, "config: unknown key loud in done of config.lua");
}

#[test]
fn a_value_of_the_wrong_type_is_refused_by_name() {
    let why =
        config("return { remote = { todoist = { url = 'todoist::', stale = 15 } } }").refusal();
    assert_eq!(
        why,
        "config: stale must be a string in remote.todoist of config.lua"
    );
    let why = config("return { owner = true }").refusal();
    assert!(why.contains("owner must be a string"), "{why}");
}

#[test]
fn tokens_and_the_schema_are_refused_as_belonging_in_secrets_lua() {
    for key in ["token = {}", "schema = 1"] {
        let why = config(&format!("return {{ {key} }}")).refusal();
        assert!(why.contains("belongs in secrets.lua"), "{why}");
    }
}

#[test]
fn a_credential_on_a_remote_is_refused_as_belonging_in_secrets_lua() {
    let why = config("return { remote = { todoist = { url = 'todoist::', api_token = 'x' } } }")
        .refusal();
    assert!(
        why.contains("unknown key api_token in remote.todoist"),
        "{why}"
    );
    assert!(why.contains("credentials belong in secrets.lua"), "{why}");
    assert!(!why.contains("'x'"), "{why}");
}

#[test]
fn a_filter_takes_only_a_query() {
    let why = config("return { filter = { today = { query = 'x', sort = 'due' } } }").refusal();
    assert_eq!(
        why,
        "config: unknown key sort in filter.today of config.lua"
    );
}

#[test]
fn a_map_where_a_list_belongs_and_a_list_where_a_map_belongs_are_refused() {
    let why = config("return { category = { effort = { values = { a = 'light' } } } }").refusal();
    assert!(why.contains("values must be a list of strings"), "{why}");
    let why = config("return { remote = { 'todoist::' } }").refusal();
    assert!(
        why.contains("remote must be a table of named entries"),
        "{why}"
    );
}

#[test]
fn a_remote_url_must_name_its_helper() {
    let why = config("return { remote = { todoist = { url = '::' } } }").refusal();
    assert!(why.contains("must look like <helper>::"), "{why}");
    let why = config("return { remote = { todoist = {} } }").refusal();
    assert!(why.contains("url is missing"), "{why}");
}

#[test]
fn a_helper_name_with_a_colon_is_refused() {
    let why = config("return { remote = { todoist = { url = 'a:b::x' } } }").refusal();
    assert!(why.contains("must look like <helper>::"), "{why}");
}

#[test]
fn a_duration_with_a_sign_is_refused() {
    let why =
        config("return { remote = { todoist = { url = 'todoist::', stale = '+15m' } } }").refusal();
    assert!(
        why.contains("stale \"+15m\" must be <number><s|m|h>"),
        "{why}"
    );
}
