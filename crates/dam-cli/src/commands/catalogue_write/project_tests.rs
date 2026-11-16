use dam_adapters::read_project_file;
use dam_application::testing::{FakeSettingsFile, MemoryStore};
use dam_application::{Config, Project};

use super::routed::{Settings, category, filter};
use crate::args::{CategoryCommand, FilterCommand};
use crate::error::CliError;
use crate::project_scope::Standing;

const PROJECT_TEXT: &str = "-- the team's rules\nreturn {\n  path = 'app/',\n}\n";

struct Machine {
    dir: tempfile::TempDir,
    personal: FakeSettingsFile,
    config: Config,
}

fn machine() -> Machine {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".dam.lua"), PROJECT_TEXT).unwrap();
    Machine {
        dir,
        personal: FakeSettingsFile::default(),
        config: Config::default(),
    }
}

impl Machine {
    fn inside(&self) -> Standing {
        let folder = self.dir.path().to_path_buf();
        let settings = read_project_file(&folder.join(".dam.lua"), "0.2.0").unwrap();
        Standing::Inside(Box::new(Project { folder, settings }))
    }

    fn settings<'a>(&'a self, standing: &'a Standing) -> Settings<'a> {
        Settings {
            config: &self.config,
            personal: &self.personal,
            standing,
        }
    }

    fn filter(&self, command: FilterCommand) -> Result<crate::output::Report, CliError> {
        filter(&self.settings(&self.inside()), command)
    }

    fn category(&self, command: CategoryCommand) -> Result<crate::output::Report, CliError> {
        category(&self.settings(&self.inside()), &MemoryStore::new(), command)
    }

    fn project_text(&self) -> String {
        std::fs::read_to_string(self.dir.path().join(".dam.lua")).unwrap()
    }
}

fn create_filter(global: bool) -> FilterCommand {
    FilterCommand::Create {
        name: "mine".into(),
        query: "due:today".into(),
        global,
    }
}

#[test]
fn a_filter_created_and_removed_inside_a_project_changes_its_file_alone() {
    let m = machine();
    let created = m.filter(create_filter(false)).unwrap();
    assert_eq!(created.data["file"], ".dam.lua");
    assert!(created.human.contains(".dam.lua"), "{}", created.human);
    assert_eq!(m.inside().filters()[0].name, "mine");
    assert!(m.project_text().starts_with("-- the team's rules\n"));
    assert!(m.personal.written.borrow().is_empty());
    let removed = FilterCommand::Remove {
        name: "mine".into(),
        global: false,
    };
    assert_eq!(m.filter(removed).unwrap().data["file"], ".dam.lua");
    assert!(m.inside().filters().is_empty());
}

#[test]
fn global_creates_the_filter_in_config_lua_and_leaves_the_project_file() {
    let m = machine();
    let created = m.filter(create_filter(true)).unwrap();
    assert_eq!(created.data["file"], "config.lua");
    assert_eq!(m.project_text(), PROJECT_TEXT);
    assert_eq!(m.personal.written.borrow().len(), 1);
}

#[test]
fn outside_a_project_the_filter_goes_to_config_lua() {
    let m = machine();
    let report = filter(&m.settings(&Standing::Outside), create_filter(false)).unwrap();
    assert_eq!(report.data["file"], "config.lua");
}

#[test]
fn a_category_created_and_removed_inside_a_project_changes_its_file_alone() {
    let m = machine();
    let created = m
        .category(CategoryCommand::Create {
            name: "size".into(),
            values: "small,large".into(),
            exclusive: true,
            global: false,
        })
        .unwrap();
    assert_eq!(created.data["file"], ".dam.lua");
    let Standing::Inside(project) = m.inside() else {
        unreachable!()
    };
    assert!(
        project
            .settings
            .categories
            .category_of_name("size")
            .is_some()
    );
    assert!(m.personal.written.borrow().is_empty());
    let removed = CategoryCommand::Remove {
        name: "size".into(),
        global: false,
    };
    assert_eq!(m.category(removed).unwrap().data["file"], ".dam.lua");
    let Standing::Inside(project) = m.inside() else {
        unreachable!()
    };
    assert!(project.settings.categories.all().is_empty());
}

#[test]
fn a_project_category_sharing_a_label_with_yours_is_refused_and_nothing_is_written() {
    let mut m = machine();
    m.config.categories = dam_domain::Categories::new(vec![dam_domain::Category {
        name: "effort".into(),
        values: vec!["small".into()],
        exclusive: false,
    }])
    .unwrap();
    let refused = m.category(CategoryCommand::Create {
        name: "size".into(),
        values: "small,large".into(),
        exclusive: false,
        global: false,
    });
    assert!(refused.is_err());
    assert_eq!(m.project_text(), PROJECT_TEXT);
}
