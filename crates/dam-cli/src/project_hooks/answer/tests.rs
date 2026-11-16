use std::path::PathBuf;

use dam_adapters::ProjectHookFolder;
use dam_application::testing::FakeProjectHookSwitches;
use dam_application::{Config, ProjectHookSwitch, ProjectHookSwitches};

use super::{ProjectHookAnswer, answer};

struct Clone {
    dir: tempfile::TempDir,
    switches: FakeProjectHookSwitches,
}

fn clone_holding(text: &str) -> Clone {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".damhooks")).unwrap();
    let clone = Clone {
        dir,
        switches: FakeProjectHookSwitches::default(),
    };
    clone.write("task_new.lua", text);
    clone
}

impl Clone {
    fn folder(&self) -> PathBuf {
        self.dir.path().to_path_buf()
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.folder().join(".damhooks").join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn turn_on(&self) {
        let read = ProjectHookFolder::read(&self.folder()).unwrap();
        let on = ProjectHookSwitch::on_for(read.fingerprint().unwrap().unwrap());
        self.switches.turn(&self.folder(), Some(on)).unwrap();
    }

    fn answer(&self, global: Option<bool>) -> &'static str {
        let personal = Config {
            project_hooks: global,
            ..Config::default()
        };
        match answer(&self.switches, &self.folder(), &personal).unwrap() {
            ProjectHookAnswer::Off => "off",
            ProjectHookAnswer::On(_) => "on",
            ProjectHookAnswer::Changed { .. } => "changed",
            ProjectHookAnswer::TooLarge => "too large",
        }
    }
}

#[test]
fn turned_on_with_the_files_unchanged_runs() {
    let clone = clone_holding("return function() end");
    clone.turn_on();
    assert_eq!(clone.answer(None), "on");
}

#[test]
fn an_edited_file_is_a_change_until_turned_on_again() {
    let clone = clone_holding("return function() end");
    clone.turn_on();
    clone.write("task_new.lua", "return function() return {} end");
    assert_eq!(clone.answer(None), "changed");
    clone.turn_on();
    assert_eq!(clone.answer(None), "on");
}

#[test]
fn an_added_file_is_a_change() {
    let clone = clone_holding("return function() end");
    clone.turn_on();
    clone.write("task_done.lua", "return function() end");
    assert_eq!(clone.answer(None), "changed");
}

#[test]
fn a_removed_or_renamed_file_is_a_change() {
    let clone = clone_holding("return function() end");
    clone.turn_on();
    let hooks = clone.folder().join(".damhooks");
    std::fs::rename(hooks.join("task_new.lua"), hooks.join("task_done.lua")).unwrap();
    assert_eq!(clone.answer(None), "changed");
    std::fs::remove_file(hooks.join("task_done.lua")).unwrap();
    assert_eq!(clone.answer(None), "changed");
}

#[test]
fn a_switch_turned_on_before_fingerprints_is_a_change() {
    let clone = clone_holding("return function() end");
    let old = ProjectHookSwitch {
        on: true,
        fingerprint: None,
    };
    clone.switches.turn(&clone.folder(), Some(old)).unwrap();
    assert_eq!(clone.answer(None), "changed");
}

#[test]
fn the_global_switch_runs_a_clone_never_turned_on_without_a_fingerprint() {
    let clone = clone_holding("return function() end");
    assert_eq!(clone.answer(Some(true)), "on");
    assert_eq!(clone.answer(None), "off");
}

#[test]
fn a_clone_whose_files_changed_does_not_run_under_the_global_switch() {
    let clone = clone_holding("return function() end");
    clone.turn_on();
    clone.write("task_new.lua", "return function() return {} end");
    assert_eq!(clone.answer(Some(true)), "changed");
}

#[test]
fn a_clone_turned_off_stays_off_under_the_global_switch() {
    let clone = clone_holding("return function() end");
    let off = ProjectHookSwitch::off();
    clone.switches.turn(&clone.folder(), Some(off)).unwrap();
    assert_eq!(clone.answer(Some(true)), "off");
}

#[test]
fn a_changed_helper_that_a_hook_requires_is_a_change() {
    let clone = clone_holding("require('lib.helpers')\nreturn function() end");
    clone.write("lib/helpers.lua", "return {}");
    clone.turn_on();
    clone.write("lib/helpers.lua", "os.execute('true')\nreturn {}");
    assert_eq!(clone.answer(None), "changed");
}

#[test]
fn a_folder_too_large_to_check_runs_no_hooks_even_under_the_global_switch() {
    let clone = clone_holding("return function() end");
    clone.turn_on();
    for n in 0..1001 {
        clone.write(&format!("lib/{n}.lua"), "");
    }
    assert_eq!(clone.answer(None), "too large");
    assert_eq!(clone.answer(Some(true)), "too large");
}
