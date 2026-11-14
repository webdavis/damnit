use std::os::unix::fs::PermissionsExt;

use dam_application::RemoteName;

use super::support::{Folder, config};

fn todoist() -> RemoteName {
    RemoteName::parse("todoist").unwrap()
}

fn added(before: &str) -> String {
    let folder = config(before);
    folder.file().add_remote(&todoist(), "todoist::").unwrap();
    assert_eq!(
        folder.load().unwrap().remote("todoist").unwrap().url,
        "todoist::"
    );
    folder.text("config.lua")
}

fn mode_of(path: &std::path::Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn a_missing_config_lua_is_written_with_just_the_remote() {
    let folder = Folder::new();
    assert_eq!(
        folder.file().add_remote(&todoist(), "todoist::").unwrap(),
        None
    );
    assert_eq!(
        folder.text("config.lua"),
        "return {\n  remote = {\n    todoist = { url = \"todoist::\" },\n  },\n}\n"
    );
}

#[test]
fn the_remote_goes_last_in_the_remote_table_and_every_comment_stays() {
    let before = "-- mine\nreturn {\n  owner = \"Ada\", -- me\n  remote = {\n    -- first\n    \
                  work = { url = \"work::\" }, -- trailing\n    -- after\n  },\n}\n-- end\n";
    assert_eq!(
        added(before),
        "-- mine\nreturn {\n  owner = \"Ada\", -- me\n  remote = {\n    -- first\n    \
         work = { url = \"work::\" }, -- trailing\n    -- after\n    \
         todoist = { url = \"todoist::\" },\n  },\n}\n-- end\n"
    );
}

#[test]
fn without_trailing_commas_the_last_entry_gains_one_and_the_new_one_has_none() {
    let before = "return {\n  remote = {\n    work = { url = \"work::\" } -- w\n  }\n}\n";
    assert_eq!(
        added(before),
        "return {\n  remote = {\n    work = { url = \"work::\" }, -- w\n    \
         todoist = { url = \"todoist::\" }\n  }\n}\n"
    );
}

#[test]
fn a_config_without_a_remote_table_gains_one() {
    assert_eq!(
        added("return {\n  owner = \"Ada\"\n}\n"),
        "return {\n  owner = \"Ada\",\n  remote = {\n    todoist = { url = \"todoist::\" },\n  }\n}\n"
    );
}

#[test]
fn an_empty_remote_table_is_filled() {
    assert_eq!(
        added("return {\n  remote = {},\n}\n"),
        "return {\n  remote = {\n    todoist = { url = \"todoist::\" },\n  },\n}\n"
    );
}

#[test]
fn a_remote_table_on_one_line_stays_on_one_line() {
    assert_eq!(
        added("return { remote = { work = { url = \"work::\" } } }\n"),
        "return { remote = { work = { url = \"work::\" }, todoist = { url = \"todoist::\" } } }\n"
    );
}

#[test]
fn the_indentation_already_there_is_matched() {
    assert_eq!(
        added("return {\n    remote = {\n        work = { url = \"work::\" },\n    },\n}\n"),
        "return {\n    remote = {\n        work = { url = \"work::\" },\n        \
         todoist = { url = \"todoist::\" },\n    },\n}\n"
    );
    assert_eq!(
        added("return {\n\towner = \"Ada\",\n}\n"),
        "return {\n\towner = \"Ada\",\n\tremote = {\n\t\ttodoist = { url = \"todoist::\" },\n\t},\n}\n"
    );
}

#[test]
fn a_remote_the_file_already_holds_is_refused_by_name_and_the_file_is_kept() {
    let before = "return { remote = { todoist = { url = \"other::\" } } }\n";
    let folder = config(before);
    let why = folder
        .file()
        .add_remote(&todoist(), "todoist::")
        .unwrap_err();
    assert_eq!(
        why.to_string(),
        "config: config.lua already holds remote todoist"
    );
    assert_eq!(folder.text("config.lua"), before);
}

#[test]
fn a_table_built_in_code_is_refused_with_the_entry_to_write_by_hand() {
    for before in [
        "local t = { remote = {} }\nreturn t\n",
        "local function remotes() return {} end\nreturn { remote = remotes() }\n",
    ] {
        let folder = config(before);
        let why = folder
            .file()
            .add_remote(&todoist(), "todoist::")
            .unwrap_err();
        let why = why.to_string();
        assert!(why.contains("built in code"), "{why}");
        assert!(why.contains("todoist = { url = \"todoist::\" }"), "{why}");
        assert_eq!(folder.text("config.lua"), before);
    }
}

#[test]
fn a_remote_the_reader_would_refuse_is_never_written() {
    let before = "return {}\n";
    let folder = config(before);
    let why = folder.file().add_remote(&todoist(), "::").unwrap_err();
    assert!(
        why.to_string().contains("must look like <helper>::"),
        "{why}"
    );
    assert_eq!(folder.text("config.lua"), before);
}

#[test]
fn config_lua_keeps_its_mode_and_a_new_one_gets_the_default_mode() {
    let folder = config("return {}\n");
    let path = folder.path("config.lua");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
    assert_eq!(
        folder.file().add_remote(&todoist(), "todoist::").unwrap(),
        None
    );
    assert_eq!(mode_of(&path), 0o640);

    let fresh = Folder::new();
    fresh.file().add_remote(&todoist(), "todoist::").unwrap();
    std::fs::write(fresh.path("plain"), "").unwrap();
    assert_eq!(
        mode_of(&fresh.path("config.lua")),
        mode_of(&fresh.path("plain"))
    );
}

#[test]
fn a_missing_folder_is_made_readable_by_its_owner_alone() {
    let base = Folder::new();
    let dir = base.path("dam");
    let file = crate::lua_config::LuaConfigFile::new(dir.join("config.lua"));
    file.add_remote(&todoist(), "todoist::").unwrap();
    assert_eq!(mode_of(&dir), 0o700);
}

#[test]
fn a_config_lua_that_is_not_lua_is_refused_with_the_line_the_reader_names() {
    let before = "return {\n  owner = = 1,\n}\n";
    let folder = config(before);
    let why = folder
        .file()
        .add_remote(&todoist(), "todoist::")
        .unwrap_err();
    assert!(why.to_string().contains("config.lua:2:"), "{why}");
    assert_eq!(folder.text("config.lua"), before);
}
