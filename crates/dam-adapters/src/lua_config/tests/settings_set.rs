use std::os::unix::fs::PermissionsExt;

use dam_application::{SettingChange, SettingKey, SettingsFile};

use super::support::{Folder, config};

fn set(folder: &Folder, key: &str, text: &str) -> SettingChange {
    let key = SettingKey::parse(key).unwrap();
    folder.file().set(&key, &key.value(text).unwrap()).unwrap()
}

fn unset(folder: &Folder, key: &str) -> SettingChange {
    folder
        .file()
        .unset(&SettingKey::parse(key).unwrap())
        .unwrap()
}

#[test]
fn a_missing_config_lua_is_written_with_just_the_setting() {
    let folder = Folder::new();
    set(&folder, "done.interactive", "true");
    assert_eq!(
        folder.text("config.lua"),
        "return {\n  done = { interactive = true },\n}\n"
    );
}

#[test]
fn a_written_value_is_replaced_and_every_comment_stays() {
    let folder = config(
        "-- mine\nreturn {\n  owner = \"Ada\", -- me\n  done = { interactive = false },\n}\n",
    );
    let change = set(&folder, "owner", "Grace");
    assert_eq!(
        change,
        SettingChange {
            key: "owner".into(),
            old: Some("Ada".into()),
            new: Some("Grace".into())
        }
    );
    set(&folder, "done.interactive", "true");
    assert_eq!(
        folder.text("config.lua"),
        "-- mine\nreturn {\n  owner = \"Grace\", -- me\n  done = { interactive = true },\n}\n"
    );
    assert_eq!(folder.load().unwrap().owner.as_deref(), Some("Grace"));
}

#[test]
fn a_new_setting_goes_into_the_table_that_holds_its_part() {
    let folder = config("return {\n  remote = {\n    work = { url = \"work::\" },\n  },\n}\n");
    set(&folder, "remote.work.deadline", "2m");
    set(&folder, "category.priority.values", "high, low");
    assert_eq!(
        folder.text("config.lua"),
        "return {\n  remote = {\n    work = { url = \"work::\", deadline = \"2m\" },\n  },\n  \
         category = { priority = { values = { \"high\", \"low\" } } },\n}\n"
    );
}

#[test]
fn unset_removes_the_setting_and_keeps_the_rest() {
    let folder =
        config("return {\n  owner = \"Ada\", -- me\n  done = { interactive = true },\n}\n");
    let change = unset(&folder, "owner");
    assert_eq!(change.old.as_deref(), Some("Ada"));
    assert_eq!(change.new, None);
    assert_eq!(
        folder.text("config.lua"),
        "return {\n  done = { interactive = true },\n}\n"
    );
}

#[test]
fn unsetting_what_is_not_written_changes_nothing() {
    let folder = Folder::new();
    assert_eq!(unset(&folder, "owner").old, None);
    assert!(!folder.path("config.lua").exists());
}

#[test]
fn the_file_mode_is_kept() {
    let folder = config("return {\n  owner = \"Ada\",\n}\n");
    let path = folder.path("config.lua");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
    set(&folder, "owner", "Grace");
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o640
    );
}

#[test]
fn a_symlinked_config_lua_is_followed_and_stays_a_link() {
    let folder = Folder::new();
    let target = folder.path("dotfiles.lua");
    std::fs::write(&target, "return {\n  owner = \"Ada\",\n}\n").unwrap();
    std::os::unix::fs::symlink(&target, folder.path("config.lua")).unwrap();
    set(&folder, "owner", "Grace");
    assert!(
        std::fs::symlink_metadata(folder.path("config.lua"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "return {\n  owner = \"Grace\",\n}\n"
    );
}

#[test]
fn a_sync_setting_goes_into_the_sync_table_of_its_remote() {
    let folder = config("return {\n  remote = {\n    work = { url = \"work::\" },\n  },\n}\n");
    set(&folder, "remote.work.sync.interval", "5m");
    set(&folder, "remote.work.sync.allowed", "false");
    let sync = folder.load().unwrap().remote("work").unwrap().sync.clone();
    assert_eq!(sync.interval.unwrap().text, "5m");
    assert_eq!(sync.allowed, Some(false));
    assert_eq!(
        unset(&folder, "remote.work.sync.allowed").old.as_deref(),
        Some("false")
    );
    assert_eq!(
        folder.text("config.lua"),
        "return {\n  remote = {\n    work = { url = \"work::\", sync = { interval = \"5m\" } },\n  },\n}\n"
    );
}
