use dam_adapters::{LuaConfigFile, read_project_file};
use dam_application::testing::FakeProjectHookSwitches;
use dam_application::{ConfigError, Project};

use crate::args::ConfigCommand;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;

const PROJECT_TEXT: &str = "-- the team's rules\nreturn {\n  path = 'app/',\n}\n";

struct Machine {
    dir: tempfile::TempDir,
    file: LuaConfigFile,
    switches: FakeProjectHookSwitches,
}

fn machine() -> Machine {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("app")).unwrap();
    std::fs::write(dir.path().join("app/.dam.lua"), PROJECT_TEXT).unwrap();
    let file = LuaConfigFile::new(dir.path().join("config.lua"));
    Machine {
        dir,
        file,
        switches: FakeProjectHookSwitches::default(),
    }
}

fn set(key: &str, value: &str, global: bool) -> ConfigCommand {
    ConfigCommand::Set {
        key: key.into(),
        value: value.into(),
        global,
    }
}

impl Machine {
    fn inside(&self) -> Standing {
        let folder = self.dir.path().join("app");
        match read_project_file(&folder.join(".dam.lua"), "0.2.0") {
            Ok(settings) => Standing::Inside(Box::new(Project { folder, settings })),
            Err(e) => Standing::Unreadable(e),
        }
    }

    fn run(&self, command: ConfigCommand, standing: &Standing) -> Result<Report, CliError> {
        super::run(&self.file, command, standing, &self.switches)
    }

    fn project_text(&self) -> String {
        std::fs::read_to_string(self.dir.path().join("app/.dam.lua")).unwrap()
    }

    fn config_text(&self) -> Option<String> {
        std::fs::read_to_string(self.dir.path().join("config.lua")).ok()
    }

    fn usage(&self, command: ConfigCommand, standing: &Standing) -> String {
        match self.run(command, standing) {
            Err(CliError::Usage(text)) => text,
            other => panic!("not refused: {:?}", other.map(|r| r.data)),
        }
    }
}

#[test]
fn a_project_key_set_inside_a_project_is_written_into_its_file_and_named() {
    let m = machine();
    let report = m.run(set("ls", "label:app", false), &m.inside()).unwrap();
    assert_eq!(report.data["file"], ".dam.lua");
    assert!(report.human.contains(".dam.lua"), "{}", report.human);
    assert!(m.project_text().starts_with("-- the team's rules\n"));
    assert_eq!(m.inside().filters().len(), 0);
    let Standing::Inside(project) = m.inside() else {
        panic!("the project file no longer loads")
    };
    assert_eq!(project.settings.ls.as_deref(), Some("label:app"));
    assert_eq!(m.config_text(), None);
}

#[test]
fn global_inside_a_project_writes_config_lua_and_leaves_the_project_file() {
    let m = machine();
    let report = m
        .run(set("filter.mine.query", "label:app", true), &m.inside())
        .unwrap();
    assert_eq!(report.data["file"], "config.lua");
    assert_eq!(m.project_text(), PROJECT_TEXT);
    assert!(m.config_text().unwrap().contains("label:app"));
}

#[test]
fn a_personal_key_inside_a_project_is_refused_naming_global_and_changes_nothing() {
    let m = machine();
    assert_eq!(
        m.usage(set("remote.todoist.path", "inbox/", false), &m.inside()),
        "remote.todoist.path is yours, not the project's; add --global to change your config.lua"
    );
    assert_eq!(m.project_text(), PROJECT_TEXT);
    assert_eq!(m.config_text(), None);
}

#[test]
fn project_hooks_never_reaches_the_project_file() {
    let m = machine();
    for global in [false, true] {
        m.run(set("project.hooks", "true", global), &m.inside())
            .unwrap();
        assert_eq!(m.project_text(), PROJECT_TEXT);
    }
}

#[test]
fn outside_a_project_a_setting_goes_to_config_lua_and_a_project_key_is_refused() {
    let m = machine();
    let report = m
        .run(set("owner", "Grace", false), &Standing::Outside)
        .unwrap();
    assert_eq!(report.data["file"], "config.lua");
    assert_eq!(
        m.usage(set("ls", "label:app", false), &Standing::Outside),
        "ls belongs to a project's .dam.lua; run dam config set inside the project's folder"
    );
    assert_eq!(
        m.usage(set("ls", "label:app", true), &m.inside()),
        "ls belongs to a project's .dam.lua; run dam config set inside the project's folder"
    );
}

#[test]
fn a_project_file_that_does_not_load_is_refused_naming_it_unless_global() {
    let m = machine();
    let unreadable = Standing::Unreadable(ConfigError::NotAPath {
        file: m.dir.path().join("app/.dam.lua"),
    });
    let refusal = m.usage(set("ls", "label:app", false), &unreadable);
    assert!(refusal.contains("app/.dam.lua"), "{refusal}");
    assert!(refusal.contains("dam doctor config"), "{refusal}");
    assert!(refusal.contains("--global"), "{refusal}");
    m.run(set("owner", "Grace", true), &unreadable).unwrap();
    assert!(m.config_text().unwrap().contains("Grace"));
}

fn unset(key: &str, global: bool) -> ConfigCommand {
    ConfigCommand::Unset {
        key: key.into(),
        global,
    }
}

#[test]
fn unset_inside_a_project_removes_the_key_from_its_file_and_refuses_path() {
    let m = machine();
    m.run(set("ls", "label:app", false), &m.inside()).unwrap();
    let report = m.run(unset("ls", false), &m.inside()).unwrap();
    assert_eq!(report.data["file"], ".dam.lua");
    assert_eq!(m.project_text(), PROJECT_TEXT);
    assert_eq!(
        m.usage(unset("path", false), &m.inside()),
        "a project needs its path; change it with dam config set path <path>"
    );
    assert_eq!(
        m.usage(unset("owner", false), &m.inside()),
        "owner is yours, not the project's; add --global to change your config.lua"
    );
    assert_eq!(m.project_text(), PROJECT_TEXT);
}
