use dam_application::{ConfigError, SettingChange, SettingKey, SettingsFile};

use super::project_support::ProjectFolder;
use crate::lua_config::project::PROJECT_SETTINGS;
use crate::lua_config::{ProjectSettingsFile, project_setting};

const EXAMPLES: [(&str, &str); 11] = [
    ("path", "work/app/"),
    ("minimum_dam", "0.1.0"),
    ("new_task.labels", "app, work"),
    ("required", "due"),
    ("list.labels", "app"),
    ("filter.mine.labels", "app"),
    ("category.size.values", "small, large"),
    ("sections", "docs"),
    ("branch_label", "true"),
    ("new_task.priority", "2"),
    ("done_children", "up"),
];

fn set(folder: &ProjectFolder, key: &str, text: &str) -> Result<SettingChange, ConfigError> {
    let key = SettingKey::parse(key).unwrap();
    let value = key.value(text).unwrap();
    ProjectSettingsFile::new(folder.file(), "0.2.0").set(&key, &value)
}

#[test]
fn every_key_the_reader_takes_is_written_and_read_back() {
    for top in PROJECT_SETTINGS {
        let (key, text) = EXAMPLES
            .iter()
            .find(|(key, _)| key.split('.').next() == Some(top))
            .unwrap_or_else(|| panic!("no example sets {top}"));
        assert!(project_setting(&SettingKey::parse(key).unwrap()), "{key}");
        let folder = ProjectFolder::with("return { path = 'app/' }");
        set(&folder, key, text).unwrap();
        assert!(folder.read().is_ok(), "{key}: {:?}", folder.read());
    }
}

#[test]
fn a_set_keeps_the_comments_and_the_other_keys_and_answers_the_old_value() {
    let text = "-- the team's rules\nreturn {\n  path = 'app/', -- where tasks go\n  list = { due = 'today' },\n}\n";
    let folder = ProjectFolder::with(text);
    let change = set(&folder, "list.due", "..today").unwrap();
    assert_eq!(change.old.as_deref(), Some("today"));
    assert_eq!(
        std::fs::read_to_string(folder.file()).unwrap(),
        text.replace("'today'", "\"..today\"")
    );
}

#[test]
fn a_value_that_would_not_load_is_refused_and_changes_nothing() {
    let text = "return { path = 'app/' }\n";
    for (key, value) in [
        ("list.due", "soon"),
        ("new_task.priority", "9"),
        ("path", "work/../app"),
        ("list.labels", "a\u{7}b"),
    ] {
        let folder = ProjectFolder::with(text);
        assert!(
            matches!(
                set(&folder, key, value),
                Err(ConfigError::ProjectSettingWouldNotLoad { .. })
            ),
            "{key} = {value:?}"
        );
        assert_eq!(std::fs::read_to_string(folder.file()).unwrap(), text);
    }
}

#[test]
fn a_personal_key_is_never_written_into_the_project_file() {
    let text = "return { path = 'app/' }\n";
    let folder = ProjectFolder::with(text);
    for key in ["project.hooks", "owner", "progress"] {
        assert!(!project_setting(&SettingKey::parse(key).unwrap()), "{key}");
        assert!(set(&folder, key, "true").is_err(), "{key}");
    }
    assert_eq!(std::fs::read_to_string(folder.file()).unwrap(), text);
}
