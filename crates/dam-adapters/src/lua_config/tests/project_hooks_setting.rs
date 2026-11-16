use dam_application::{SettingKey, SettingsFile};

use super::support::{Folder, config};

#[test]
fn project_hooks_in_config_lua_turns_every_project_on() {
    let on = config("return { project = { hooks = true } }\n")
        .load()
        .unwrap();
    assert_eq!(on.project_hooks, Some(true));
    assert_eq!(Folder::new().load().unwrap().project_hooks, None);
}

#[test]
fn dam_config_set_writes_project_hooks_into_config_lua() {
    let folder = Folder::new();
    let key = SettingKey::parse("project.hooks").unwrap();
    folder
        .file()
        .set(&key, &key.value("true").unwrap())
        .unwrap();
    assert_eq!(folder.load().unwrap().project_hooks, Some(true));
}
