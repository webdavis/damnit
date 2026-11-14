use dam_application::{ByHand, ConfigError, ConfigFile, EditFault, EditedTable, RemoteName};

use super::support::{Folder, bare_token, config, held_already, secrets, value};

fn todoist() -> RemoteName {
    RemoteName::parse("todoist").unwrap()
}

#[test]
fn a_token_under_a_key_written_another_way_is_held_already() {
    for key in [r#"["to\x6fl"]"#, r#"["to" .. "ol"]"#] {
        let before = format!("return {{ token = {{ {key} = {{ value = \"x\" }} }} }}\n");
        let folder = secrets(&before);
        let why = folder.file().add_token(&bare_token("tool")).unwrap_err();
        assert_eq!(why, held_already(EditedTable::Tokens, "tool"));
        assert_eq!(folder.text("secrets.lua"), before);
    }
}

#[test]
fn a_remote_under_a_key_written_another_way_is_held_already() {
    let before = "return { remote = { [\"todo\\x69st\"] = { url = \"work::\" } } }\n";
    let folder = config(before);
    let why = folder
        .file()
        .add_remote(&todoist(), "todoist::")
        .unwrap_err();
    assert_eq!(why, held_already(EditedTable::Remotes, "todoist"));
    assert_eq!(folder.text("config.lua"), before);
}

#[test]
fn a_token_under_a_key_written_another_way_is_left_for_a_hand_edit() {
    let before = "return { token = { [\"to\\x6fl\"] = { value = \"x\" } } }\n";
    let folder = secrets(before);
    let why = folder.file().remove_token("tool").unwrap_err();
    assert_eq!(
        why,
        ConfigError::HeldUnderUneditableKey {
            table: EditedTable::Tokens,
            entry: "tool".into(),
            by_hand: ByHand::RemoveToken {
                name: "tool".into()
            },
        }
    );
    assert_eq!(folder.text("secrets.lua"), before);
}

#[test]
fn a_token_edit_that_would_hide_the_tokens_held_is_refused_and_the_file_kept() {
    let before = format!(
        "return {{ [\"tok\" .. \"en\"] = {{ laptop = {{ value = {:?} }} }} }}\n",
        value(1)
    );
    let folder = secrets(&before);
    let why = folder.file().add_token(&bare_token("tool")).unwrap_err();
    assert_eq!(
        why,
        would_break(
            EditedTable::Tokens,
            "tool",
            EditFault::WouldNotKeepTheOthers
        )
    );
    assert_eq!(folder.text("secrets.lua"), before);
}

#[test]
fn a_remote_edit_that_would_hide_the_remotes_held_is_refused_and_the_file_kept() {
    let before = "return { [\"rem\" .. \"ote\"] = { work = { url = \"work::\" } } }\n";
    let folder = config(before);
    let why = folder
        .file()
        .add_remote(&todoist(), "todoist::")
        .unwrap_err();
    assert_eq!(
        why,
        would_break(
            EditedTable::Remotes,
            "todoist",
            EditFault::WouldNotKeepTheOthers
        )
    );
    assert_eq!(folder.text("config.lua"), before);
}

#[test]
fn a_removal_that_would_leave_the_token_held_is_refused_and_the_file_kept() {
    let before =
        "return { token = { tool = { value = \"x\" }, [\"to\\x6fl\"] = { value = \"y\" } } }\n";
    let folder = secrets(before);
    let why = folder.file().remove_token("tool").unwrap_err();
    assert_eq!(
        why,
        would_break(
            EditedTable::Tokens,
            "tool",
            EditFault::WouldStillHoldTheEntry
        )
    );
    assert_eq!(folder.text("secrets.lua"), before);
}

#[test]
fn a_token_edit_reads_secrets_lua_alone() {
    let folder = Folder::new()
        .with_config("return {\n  owner = = 1,\n}\n")
        .with_secrets("return { remote = { work = { api_token = 5 } } }\n");
    folder.file().add_token(&bare_token("tool")).unwrap();
    folder.file().remove_token("tool").unwrap();
    assert!(!folder.text("secrets.lua").contains("tool"));
}

#[test]
fn adding_a_remote_reads_config_lua_alone() {
    let folder = Folder::new().with_secrets("return { remote = = 1 }\n");
    folder.file().add_remote(&todoist(), "todoist::").unwrap();
    assert!(folder.text("config.lua").contains("todoist::"));
}

fn would_break(table: EditedTable, entry: &str, fault: EditFault) -> ConfigError {
    ConfigError::EditWouldBreak {
        table,
        entry: entry.into(),
        fault,
    }
}
