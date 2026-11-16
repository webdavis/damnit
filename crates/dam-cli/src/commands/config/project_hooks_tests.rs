use std::path::PathBuf;

use dam_adapters::LuaConfigFile;
use dam_application::testing::FakeProjectHookSwitches;
use dam_application::{ConfigFile, Project, ProjectHookSwitches, ProjectSettings};

use crate::args::ConfigCommand;
use crate::project_scope::Standing;

struct Machine {
    _dir: tempfile::TempDir,
    file: LuaConfigFile,
    switches: FakeProjectHookSwitches,
}

fn machine() -> Machine {
    let dir = tempfile::tempdir().unwrap();
    let file = LuaConfigFile::new(dir.path().join("config.lua"));
    Machine {
        _dir: dir,
        file,
        switches: FakeProjectHookSwitches::default(),
    }
}

fn clone() -> PathBuf {
    PathBuf::from("/work/app")
}

fn inside() -> Standing {
    Standing::Inside(Box::new(Project {
        folder: clone(),
        settings: ProjectSettings {
            path: "app/".into(),
            ..Default::default()
        },
    }))
}

fn set(global: bool) -> ConfigCommand {
    ConfigCommand::Set {
        key: "project.hooks".into(),
        value: "true".into(),
        global,
    }
}

impl Machine {
    fn run(&self, command: ConfigCommand, standing: &Standing) -> crate::output::Report {
        super::run(&self.file, command, standing, &self.switches).unwrap()
    }

    fn personal(&self) -> Option<bool> {
        self.file.load().unwrap().config.project_hooks
    }
}

#[test]
fn set_inside_a_project_turns_on_this_clone_and_leaves_config_lua_alone() {
    let m = machine();
    m.run(set(false), &inside());
    assert_eq!(m.switches.switch(&clone()), Ok(Some(true)));
    assert_eq!(m.personal(), None);
}

#[test]
fn set_global_inside_a_project_writes_config_lua_and_no_clone() {
    let m = machine();
    m.run(set(true), &inside());
    assert_eq!(m.switches.switch(&clone()), Ok(None));
    assert_eq!(m.personal(), Some(true));
}

#[test]
fn set_outside_a_project_writes_config_lua() {
    let m = machine();
    m.run(set(false), &Standing::Outside);
    assert_eq!(m.personal(), Some(true));
}

#[test]
fn unset_inside_a_project_forgets_this_clone() {
    let m = machine();
    m.run(set(false), &inside());
    let unset = ConfigCommand::Unset {
        key: "project.hooks".into(),
        global: false,
    };
    let changed = m.run(unset, &inside());
    assert_eq!(m.switches.switch(&clone()), Ok(None));
    assert_eq!(changed.data["old"], "true");
}

#[test]
fn get_inside_a_project_reads_this_clone_first() {
    let m = machine();
    m.run(set(false), &inside());
    let get = || ConfigCommand::Get {
        key: "project.hooks".into(),
    };
    assert_eq!(m.run(get(), &inside()).data["value"], true);
    assert_eq!(m.run(get(), &Standing::Outside).data["value"], false);
}

#[test]
fn the_list_inside_a_project_says_whether_its_hooks_are_on_and_from_where() {
    let m = machine();
    let listed = || m.run(ConfigCommand::List, &inside());
    assert_eq!(listed().data["project"]["hooks"], false);
    assert_eq!(listed().data["project"]["hooks_from"], "default");
    m.run(set(true), &inside());
    assert_eq!(listed().data["project"]["hooks_from"], "config.lua");
    m.run(set(false), &inside());
    assert_eq!(listed().data["project"]["hooks"], true);
    assert_eq!(listed().data["project"]["hooks_from"], "this clone");
    assert!(
        listed().human.contains("on (from this clone)"),
        "{}",
        listed().human
    );
}
