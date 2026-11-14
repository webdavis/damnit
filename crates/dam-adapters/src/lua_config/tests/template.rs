use dam_domain::Permission;

use super::support::{Folder, config};

const TEMPLATE: &str = include_str!("../template.lua");

fn is_example(line: &str) -> bool {
    line.trim_start()
        .strip_prefix("--")
        .is_some_and(|rest| !rest.is_empty() && !rest.starts_with([' ', '-']))
}

fn uncommented(line: &str) -> String {
    match is_example(line) {
        true => line.replacen("--", "", 1),
        false => line.to_string(),
    }
}

#[test]
fn a_missing_config_lua_is_made_from_the_template_in_a_private_folder() {
    let base = Folder::new();
    let file = crate::lua_config::LuaConfigFile::new(base.path("dam/config.lua"));
    assert!(file.create_if_missing().unwrap());
    assert_eq!(base.text("dam/config.lua"), TEMPLATE);
    let mode = std::os::unix::fs::PermissionsExt::mode(
        &std::fs::metadata(base.path("dam")).unwrap().permissions(),
    );
    assert_eq!(mode & 0o777, 0o700);
}

#[test]
fn a_config_lua_already_there_is_kept() {
    let folder = config("return { owner = \"Ada\" }\n");
    assert!(!folder.file().create_if_missing().unwrap());
    assert_eq!(folder.text("config.lua"), "return { owner = \"Ada\" }\n");
}

#[test]
fn the_template_reads_as_an_empty_config() {
    assert_eq!(config(TEMPLATE).load().unwrap(), Default::default());
}

#[test]
fn every_example_uncommented_at_once_is_a_working_config() {
    let text: Vec<String> = TEMPLATE.lines().map(uncommented).collect();
    let config = config(&text.join("\n")).load().unwrap();
    assert_eq!(config.owner_name(), "Ada");
    assert!(config.done_interactive);
    let remote = config.remote("todoist").unwrap();
    assert_eq!(remote.url, "todoist::");
    assert!(remote.stale.is_some() && remote.deadline.is_some() && remote.path.is_some());
    let effort = config.categories.category_of_name("effort").unwrap();
    assert_eq!(effort.values, ["light", "admin", "deep"]);
    assert!(effort.exclusive);
    assert_eq!(config.filter("today").unwrap().query, "due:today | overdue");
}

#[test]
fn every_permission_dam_grants_is_named() {
    for permission in Permission::ALL {
        assert!(TEMPLATE.contains(permission.as_str()), "{permission:?}");
    }
}

#[test]
fn the_template_explains_secrets_lua_and_never_mentions_toml() {
    assert!(TEMPLATE.contains("secrets.lua"));
    assert!(!TEMPLATE.to_lowercase().contains("toml"));
}
