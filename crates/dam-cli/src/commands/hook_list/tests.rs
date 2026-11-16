use dam_application::{ConfigError, HookEvent, HookFile, HookSetting, OnFailure, PulledChanges};
use serde_json::json;

use super::listed;

fn loaded(event: HookEvent, setting: HookSetting) -> HookFile {
    HookFile {
        file: format!("hooks/{}.lua", event.as_str()),
        event: Some(event),
        read: Ok(setting),
    }
}

#[test]
fn each_hook_shows_its_event_file_and_setting_marking_a_default() {
    let report = listed(vec![
        loaded(
            HookEvent::TaskNew,
            HookSetting::OnFailure(Some(OnFailure::Refuse)),
        ),
        loaded(HookEvent::TaskDone, HookSetting::OnFailure(None)),
        loaded(
            HookEvent::TaskPulled,
            HookSetting::Changes(Some(PulledChanges::Working)),
        ),
        loaded(HookEvent::TaskPulled, HookSetting::Changes(None)),
    ])
    .unwrap();
    assert_eq!(
        report.human,
        "task_new     hooks/task_new.lua     on_failure  refuse\n\
         task_done    hooks/task_done.lua    on_failure  warn (default)\n\
         task_pulled  hooks/task_pulled.lua  changes     working\n\
         task_pulled  hooks/task_pulled.lua  changes     committed (default)"
    );
    assert_eq!(
        report.data["hooks"],
        json!([
            {"hook": "task_new", "file": "hooks/task_new.lua", "loads": true, "on_failure": "refuse"},
            {"hook": "task_done", "file": "hooks/task_done.lua", "loads": true, "on_failure": "warn"},
            {"hook": "task_pulled", "file": "hooks/task_pulled.lua", "loads": true, "changes": "working"},
            {"hook": "task_pulled", "file": "hooks/task_pulled.lua", "loads": true, "changes": "committed"},
        ])
    );
}

#[test]
fn a_hook_that_does_not_load_says_why() {
    let report = listed(vec![HookFile {
        file: "hooks/task_neww.lua".into(),
        event: None,
        read: Err(ConfigError::UnknownHookFile {
            file: "hooks/task_neww.lua".into(),
        }),
    }])
    .unwrap();
    let why = "hooks/task_neww.lua names no event; a hook file is named task_new.lua, \
               task_changed.lua, task_done.lua or task_pulled.lua";
    assert_eq!(report.human, "-  hooks/task_neww.lua  does not load");
    assert_eq!(report.standard_error_lines, [format!("warning: {why}")]);
    assert_eq!(
        report.data["hooks"],
        json!([{"hook": null, "file": "hooks/task_neww.lua", "loads": false, "error": why}])
    );
}

#[test]
fn no_hook_files_says_so_and_answers_an_empty_list() {
    let report = listed(Vec::new()).unwrap();
    assert_eq!(report.human, "no hooks in the hooks folder");
    assert_eq!(report.data, json!({"hooks": []}));
}
