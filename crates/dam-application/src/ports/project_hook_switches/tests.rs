use super::ProjectHookSwitch;
use crate::HookFingerprint;

fn files(text: &str) -> HookFingerprint {
    HookFingerprint::of([("task_new.lua", text.as_bytes())])
}

#[test]
fn turned_on_for_the_files_as_they_are_is_not_changed() {
    let switch = ProjectHookSwitch::on_for(files("a"));
    assert!(!switch.changed_since_turned_on(&files("a")));
}

#[test]
fn turned_on_for_other_bytes_is_changed() {
    let switch = ProjectHookSwitch::on_for(files("a"));
    assert!(switch.changed_since_turned_on(&files("b")));
}

#[test]
fn turned_on_before_fingerprints_existed_is_changed() {
    let switch = ProjectHookSwitch {
        on: true,
        fingerprint: None,
    };
    assert!(switch.changed_since_turned_on(&files("a")));
}

#[test]
fn turned_off_is_never_changed() {
    assert!(!ProjectHookSwitch::off().changed_since_turned_on(&files("a")));
}
