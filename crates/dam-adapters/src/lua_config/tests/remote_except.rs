use dam_application::{ConfigError, Misread, Place, SettingKey, SettingsFile};
use dam_domain::Path;

use super::project_support::ProjectFolder;
use super::support::{Folder, config};

fn remote_with(except: &str) -> Folder {
    config(&format!(
        "return {{ remote = {{ todoist = {{ url = \"todoist::\", except = {except} }} }} }}"
    ))
}

#[test]
fn except_is_read_as_a_list_of_paths() {
    let config = remote_with(r#"{ "kitchen/", "journal/private/" }"#)
        .load()
        .unwrap();
    assert_eq!(
        config.remote("todoist").unwrap().except,
        vec![
            Path::parse("kitchen/").unwrap(),
            Path::parse("journal/private/").unwrap()
        ]
    );
}

#[test]
fn an_empty_except_is_the_same_as_none() {
    let config = remote_with("{}").load().unwrap();
    assert!(config.remote("todoist").unwrap().except.is_empty());
}

#[test]
fn an_except_that_is_not_a_path_is_refused_naming_the_key() {
    let ConfigError::Misread(misread) = remote_with(r#"{ "a//b" }"#).load().unwrap_err() else {
        panic!("not refused as a misread");
    };
    let Misread::NotAPath { text, at, .. } = *misread else {
        panic!("not refused as a path");
    };
    assert_eq!(text, "a//b");
    assert_eq!(
        at,
        Place {
            file: "config.lua".into(),
            table: Some("remote.todoist.except".into())
        }
    );
}

#[test]
fn set_and_unset_write_except_as_a_list() {
    let folder =
        config("return {\n  remote = {\n    todoist = { url = \"todoist::\" },\n  },\n}\n");
    let key = SettingKey::parse("remote.todoist.except").unwrap();
    let value = key.value("kitchen, journal/private").unwrap();
    folder.file().set(&key, &value).unwrap();
    assert_eq!(
        folder.load().unwrap().remote("todoist").unwrap().except,
        vec![
            Path::parse("kitchen").unwrap(),
            Path::parse("journal/private").unwrap()
        ]
    );
    folder.file().unset(&key).unwrap();
    assert!(
        folder
            .load()
            .unwrap()
            .remote("todoist")
            .unwrap()
            .except
            .is_empty()
    );
}

#[test]
fn a_project_file_can_never_name_what_a_remote_receives() {
    for text in [
        "return { path = 'a', remote = { todoist = { except = { 'kitchen' } } } }",
        "return { path = 'a', except = { 'kitchen' } }",
    ] {
        assert!(ProjectFolder::with(text).read().is_err(), "{text}");
    }
}
