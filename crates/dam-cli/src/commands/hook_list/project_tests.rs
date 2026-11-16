use std::path::PathBuf;

use dam_adapters::{LuaConfigFile, ProjectHookFolder};
use dam_application::testing::FakeProjectHookSwitches;
use dam_application::{Project, ProjectHookSwitch, ProjectHookSwitches, ProjectSettings};
use serde_json::json;

use super::{ProjectHookLook, run};
use crate::project_scope::Standing;

struct Machine {
    dir: tempfile::TempDir,
    switches: FakeProjectHookSwitches,
}

fn machine() -> Machine {
    let dir = tempfile::tempdir().unwrap();
    let hooks = dir.path().join("app").join(".damhooks");
    std::fs::create_dir_all(&hooks).unwrap();
    let marker = dir.path().join("loaded");
    std::fs::write(
        hooks.join("task_new.lua"),
        format!("io.open({marker:?}, 'w'):close()\nreturn function() end\n"),
    )
    .unwrap();
    Machine {
        dir,
        switches: FakeProjectHookSwitches::default(),
    }
}

impl Machine {
    fn app(&self) -> PathBuf {
        self.dir.path().join("app")
    }

    fn listed(&self) -> crate::output::Report {
        let file = LuaConfigFile::new(self.dir.path().join("config.lua"));
        let standing = Standing::Inside(Box::new(Project {
            folder: self.app(),
            settings: ProjectSettings {
                path: "app/".into(),
                ..Default::default()
            },
        }));
        let look = ProjectHookLook {
            standing: &standing,
            switches: &self.switches,
        };
        run(&file, &look).unwrap()
    }

    fn turn_on(&self) {
        let fingerprint = ProjectHookFolder::read(&self.app()).unwrap().fingerprint();
        let on = ProjectHookSwitch::on_for(fingerprint.unwrap().unwrap());
        self.switches.turn(&self.app(), Some(on)).unwrap();
    }

    fn hook(&self, name: &str) -> PathBuf {
        self.app().join(".damhooks").join(name)
    }

    fn loaded(&self) -> bool {
        self.dir.path().join("loaded").exists()
    }
}

#[test]
fn with_project_hooks_off_the_list_names_the_files_without_loading_them() {
    let m = machine();
    let report = m.listed();
    assert!(!m.loaded());
    assert_eq!(
        report.data["hooks"],
        json!([{"hook": "task_new", "file": ".damhooks/task_new.lua", "project": true, "off": true}])
    );
    assert_eq!(report.human, "task_new  .damhooks/task_new.lua  off");
    assert_eq!(
        report.standard_error_lines,
        ["warning: this project's hooks are off; dam config set project.hooks true turns them on"]
    );
}

#[test]
fn with_project_hooks_on_the_list_loads_them_marked_as_the_projects() {
    let m = machine();
    m.turn_on();
    let report = m.listed();
    assert!(m.loaded());
    assert_eq!(
        report.data["hooks"],
        json!([{"hook": "task_new", "file": ".damhooks/task_new.lua", "project": true, "loads": true, "on_failure": "warn"}])
    );
}

#[test]
fn with_hook_files_changed_since_turned_on_the_list_names_each_change_without_loading() {
    let m = machine();
    std::fs::write(m.hook("task_done.lua"), "return function() end").unwrap();
    std::fs::write(m.hook("task_changed.lua"), "return function() end").unwrap();
    m.turn_on();
    std::fs::write(m.hook("task_new.lua"), "return function() return {} end").unwrap();
    std::fs::remove_file(m.hook("task_done.lua")).unwrap();
    std::fs::write(m.hook("task_pulled.lua"), "return function() end").unwrap();
    std::fs::remove_file(m.dir.path().join("loaded")).ok();
    let report = m.listed();
    assert!(!m.loaded());
    assert_eq!(
        report.data["hooks"],
        json!([
            {"hook": "task_new", "file": ".damhooks/task_new.lua", "project": true, "changed": "changed"},
            {"hook": "task_changed", "file": ".damhooks/task_changed.lua", "project": true, "changed": "unchanged"},
            {"hook": "task_pulled", "file": ".damhooks/task_pulled.lua", "project": true, "changed": "new"},
            {"hook": null, "file": ".damhooks/task_done.lua", "project": true, "changed": "removed"},
        ])
    );
    assert_eq!(
        report.standard_error_lines,
        [
            "warning: this project's hooks changed since you turned them on, so they did not run; dam hook list shows them, and dam config set project.hooks true turns them on again"
        ]
    );
}

#[test]
fn with_helpers_changed_since_turned_on_the_list_names_each_entry_that_differs() {
    let m = machine();
    std::fs::create_dir_all(m.hook("lib")).unwrap();
    for helper in ["lib/helpers.lua", "lib/same.lua", "lib/gone.lua"] {
        std::fs::write(m.hook(helper), "return {}").unwrap();
    }
    m.turn_on();
    std::fs::write(m.hook("lib/helpers.lua"), "return { x = 1 }").unwrap();
    std::fs::remove_file(m.hook("lib/gone.lua")).unwrap();
    std::fs::create_dir(m.hook("lib/sub")).unwrap();
    std::fs::write(m.hook("lib/sub/new.txt"), "").unwrap();
    let report = m.listed();
    let other = |file: &str, changed: &str| json!({"hook": null, "file": file, "project": true, "changed": changed});
    assert_eq!(
        report.data["hooks"],
        json!([
            {"hook": "task_new", "file": ".damhooks/task_new.lua", "project": true, "changed": "unchanged"},
            other(".damhooks/lib/gone.lua", "removed"),
            other(".damhooks/lib/helpers.lua", "changed"),
            other(".damhooks/lib/sub/", "new"),
            other(".damhooks/lib/sub/new.txt", "new"),
        ])
    );
    assert_eq!(
        report.human,
        [
            "task_new  .damhooks/task_new.lua     unchanged",
            "-         .damhooks/lib/gone.lua     removed",
            "-         .damhooks/lib/helpers.lua  changed",
            "-         .damhooks/lib/sub/         new",
            "-         .damhooks/lib/sub/new.txt  new",
        ]
        .join("\n")
    );
}
