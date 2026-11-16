use dam_adapters::LuaConfigFile;

use crate::error::CliError;
use crate::output::{Format, shown};

fn config_folder() -> (tempfile::TempDir, LuaConfigFile) {
    let dir = tempfile::tempdir().unwrap();
    let file = LuaConfigFile::new(dir.path().join("config.lua"));
    (dir, file)
}

#[test]
fn a_set_setting_is_what_get_reads() {
    let (_dir, file) = config_folder();
    super::set::run(&file, "progress", Some("false")).unwrap();
    let got = super::get::run(&file, "progress").unwrap();
    assert_eq!(
        got.data,
        serde_json::json!({"value": false, "default": false})
    );
}

#[test]
fn confirm_above_is_set_as_a_whole_number_and_unset_until_then() {
    let (_dir, file) = config_folder();
    let unset = super::get::run(&file, "confirm_above").unwrap();
    assert_eq!(unset.data["value"], serde_json::Value::Null);
    super::set::run(&file, "confirm_above", Some("10")).unwrap();
    let got = super::get::run(&file, "confirm_above").unwrap();
    assert_eq!(got.data, serde_json::json!({"value": 10, "default": false}));
    assert!(super::set::run(&file, "confirm_above", Some("many")).is_err());
}

#[test]
fn quiet_prints_nothing_for_a_set() {
    let (_dir, file) = config_folder();
    let report = super::set::run(&file, "owner", Some("Grace")).unwrap();
    assert_eq!(shown(Format::Quiet, false, None, &report).unwrap(), None);
    assert!(report.standard_error_lines.is_empty());
}

#[test]
fn the_sync_settings_of_a_remote_show_their_defaults_and_are_set_and_unset_by_key() {
    let (_dir, file) = config_folder();
    super::set::run(&file, "remote.todoist.url", Some("todoist::")).unwrap();
    let got = |key: &str| super::get::run(&file, key).unwrap().data;
    assert_eq!(
        got("remote.todoist.sync"),
        serde_json::json!({
            "interval": {"value": "15m", "default": true},
            "allowed": {"value": true, "default": true},
        })
    );
    super::set::run(&file, "remote.todoist.sync.interval", Some("5m")).unwrap();
    super::set::run(&file, "remote.todoist.sync.allowed", Some("false")).unwrap();
    assert_eq!(
        got("remote.todoist.sync.interval"),
        serde_json::json!({"value": "5m", "default": false})
    );
    assert_eq!(
        got("remote.todoist.sync.allowed"),
        serde_json::json!({"value": false, "default": false})
    );
    super::set::run(&file, "remote.todoist.sync.allowed", None).unwrap();
    assert_eq!(
        got("remote.todoist.sync.allowed"),
        serde_json::json!({"value": true, "default": true})
    );
}

fn refusal(key: &str, value: &str) -> String {
    let (_dir, file) = config_folder();
    match super::set::run(&file, key, Some(value)) {
        Err(CliError::Usage(text)) => text,
        _ => panic!("{key} = {value} was not refused"),
    }
}

#[test]
fn a_sync_interval_under_a_minute_is_refused_naming_the_floor() {
    assert_eq!(
        refusal("remote.todoist.sync.interval", "30s"),
        "remote.todoist.sync.interval takes a duration of at least 1m, and \"30s\" is not one"
    );
}

#[test]
fn a_remote_key_that_is_no_setting_is_refused_without_calling_it_a_secret() {
    for key in ["remote.todoist.sync_every", "remote.todoist.stale"] {
        assert_eq!(
            refusal(key, "15m"),
            format!(
                "{key} is not a setting of config.lua; a credential belongs in secrets.lua, \
                 and dam config set never writes one"
            )
        );
    }
}

#[test]
fn a_hook_key_is_refused_naming_the_file_the_hook_goes_in() {
    assert_eq!(
        refusal("hook.task_done.on_failure", "refuse"),
        "hook.task_done.on_failure is part of a hook; write the hook by hand in \
         hooks/task_done.lua beside config.lua"
    );
}
