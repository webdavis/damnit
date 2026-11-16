use std::path::PathBuf;

use dam_adapters::LuaConfigFile;
use dam_application::testing::FakeProjectHookSwitches;
use dam_application::{Project, ProjectHookSwitches, ProjectSettings};
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
    m.switches.turn(&m.app(), Some(true)).unwrap();
    let report = m.listed();
    assert!(m.loaded());
    assert_eq!(
        report.data["hooks"],
        json!([{"hook": "task_new", "file": ".damhooks/task_new.lua", "project": true, "loads": true, "on_failure": "warn"}])
    );
}
