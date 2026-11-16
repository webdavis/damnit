use std::path::PathBuf;

use dam_adapters::{LuaConfigFile, ProjectHookFolder};
use dam_application::testing::FakeProjectHookSwitches;
use dam_application::{
    ConfigFile, Project, ProjectHookSwitch, ProjectHookSwitches, ProjectSettings,
};

use crate::args::ConfigCommand;
use crate::project_scope::Standing;

struct Machine {
    dir: tempfile::TempDir,
    file: LuaConfigFile,
    switches: FakeProjectHookSwitches,
}

fn machine() -> Machine {
    let dir = tempfile::tempdir().unwrap();
    let file = LuaConfigFile::new(dir.path().join("config.lua"));
    std::fs::create_dir_all(dir.path().join("app").join(".damhooks")).unwrap();
    let m = Machine {
        dir,
        file,
        switches: FakeProjectHookSwitches::default(),
    };
    m.write_hook("return function() end");
    m
}

fn set_to(value: &str) -> ConfigCommand {
    ConfigCommand::Set {
        key: "project.hooks".into(),
        value: value.into(),
        global: false,
    }
}

impl Machine {
    fn clone(&self) -> PathBuf {
        self.dir.path().join("app")
    }

    fn write_hook(&self, text: &str) {
        let hook = self.clone().join(".damhooks").join("task_new.lua");
        std::fs::write(hook, text).unwrap();
    }

    fn as_they_are(&self) -> ProjectHookSwitch {
        let fingerprint = ProjectHookFolder::read(&self.clone())
            .unwrap()
            .fingerprint();
        ProjectHookSwitch::on_for(fingerprint.unwrap().unwrap())
    }

    fn inside(&self) -> Standing {
        inside_of(self.clone())
    }
}

fn inside_of(folder: PathBuf) -> Standing {
    Standing::Inside(Box::new(Project {
        folder,
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
    m.run(set(false), &m.inside());
    assert_eq!(m.switches.switch(&m.clone()), Ok(Some(m.as_they_are())));
    assert_eq!(m.personal(), None);
}

#[test]
fn set_global_inside_a_project_writes_config_lua_and_no_clone() {
    let m = machine();
    m.run(set(true), &m.inside());
    assert_eq!(m.switches.switch(&m.clone()), Ok(None));
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
    m.run(set(false), &m.inside());
    let unset = ConfigCommand::Unset {
        key: "project.hooks".into(),
        global: false,
    };
    let changed = m.run(unset, &m.inside());
    assert_eq!(m.switches.switch(&m.clone()), Ok(None));
    assert_eq!(changed.data["old"], "true");
}

#[test]
fn get_inside_a_project_reads_this_clone_first() {
    let m = machine();
    m.run(set(false), &m.inside());
    let get = || ConfigCommand::Get {
        key: "project.hooks".into(),
    };
    assert_eq!(m.run(get(), &m.inside()).data["value"], true);
    assert_eq!(m.run(get(), &Standing::Outside).data["value"], false);
}

#[test]
fn the_list_inside_a_project_says_whether_its_hooks_are_on_and_from_where() {
    let m = machine();
    let listed = || m.run(ConfigCommand::List, &m.inside());
    assert_eq!(listed().data["project"]["hooks"], false);
    assert_eq!(listed().data["project"]["hooks_from"], "default");
    m.run(set(true), &m.inside());
    assert_eq!(listed().data["project"]["hooks_from"], "config.lua");
    m.run(set(false), &m.inside());
    assert_eq!(listed().data["project"]["hooks"], true);
    assert_eq!(listed().data["project"]["hooks_from"], "this clone");
    assert!(
        listed().human.contains("on (from this clone)"),
        "{}",
        listed().human
    );
}

#[test]
fn turning_on_again_records_the_files_as_they_are_now() {
    let m = machine();
    m.run(set(false), &m.inside());
    m.write_hook("return function() return { labels = { 'x' } } end");
    m.run(set(false), &m.inside());
    assert_eq!(m.switches.switch(&m.clone()), Ok(Some(m.as_they_are())));
}

#[test]
fn turning_off_keeps_no_fingerprint() {
    let m = machine();
    m.run(set(false), &m.inside());
    m.run(set_to("false"), &m.inside());
    assert_eq!(
        m.switches.switch(&m.clone()),
        Ok(Some(ProjectHookSwitch::off()))
    );
}

#[test]
fn the_list_says_when_this_clones_hooks_changed_since_turned_on() {
    let m = machine();
    m.run(set(false), &m.inside());
    m.write_hook("return function() return { labels = { 'x' } } end");
    let listed = m.run(ConfigCommand::List, &m.inside());
    assert_eq!(listed.data["project"]["hooks"], false);
    assert_eq!(listed.data["project"]["hooks_changed"], true);
    assert!(
        listed.human.contains("changed since turned on"),
        "{}",
        listed.human
    );
}
