use dam_application::RemoteName;

use super::support::{Folder, bare_token, config, secrets, value};

fn todoist() -> RemoteName {
    RemoteName::parse("todoist").unwrap()
}

#[test]
fn a_token_under_a_key_written_another_way_is_held_already() {
    for key in [r#"["to\x6fl"]"#, r#"["to" .. "ol"]"#] {
        let before = format!("return {{ token = {{ {key} = {{ value = \"x\" }} }} }}\n");
        let folder = secrets(&before);
        let why = folder.file().add_token(&bare_token("tool")).unwrap_err();
        assert_eq!(
            why.to_string(),
            "config: secrets.lua already holds token tool"
        );
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
    assert_eq!(
        why.to_string(),
        "config: config.lua already holds remote todoist"
    );
    assert_eq!(folder.text("config.lua"), before);
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
        why.to_string(),
        "config: the new secrets.lua would not keep every other token"
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
        why.to_string(),
        "config: the new config.lua would not keep every other remote"
    );
    assert_eq!(folder.text("config.lua"), before);
}

#[test]
fn adding_a_remote_reads_config_lua_alone() {
    let folder = Folder::new().with_secrets("return { remote = = 1 }\n");
    folder.file().add_remote(&todoist(), "todoist::").unwrap();
    assert!(folder.text("config.lua").contains("todoist::"));
}
