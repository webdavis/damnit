use dam_application::{ConfigError, Project, ProjectSettings};
use dam_domain::{Categories, Category};

use super::super::config_check::check;
use super::super::machine::Machine;
use super::Files;
use crate::project_scope::Standing;
use dam_application::NoProgress;

#[test]
fn a_project_file_that_does_not_load_is_a_config_finding() {
    let files = Files::new("return {}", "return {}");
    let store = files.store_path();
    let launcher = super::Helpers::default();
    let unreadable = Standing::Unreadable(ConfigError::NotATable {
        path: "/work/damnit/.dam.lua".into(),
        returned: "a number".into(),
    });
    let machine = Machine::new(&files.config, &store, &launcher, &NoProgress).within(unreadable);
    let found = serde_json::to_value(check(&machine)).unwrap();
    let project = &found[1];
    assert_eq!(project["name"], "config");
    assert_eq!(project["result"], "issue");
    assert!(
        project["detail"]
            .as_str()
            .unwrap()
            .contains("/work/damnit/.dam.lua")
    );
    assert_eq!(project["advice"], "fix the file it names");
}

#[test]
fn a_project_file_that_loads_adds_no_finding() {
    let files = Files::new("return {}", "return {}");
    let store = files.store_path();
    let launcher = super::Helpers::default();
    let machine = Machine::new(&files.config, &store, &launcher, &NoProgress);
    assert_eq!(check(&machine).len(), 1);
}

#[test]
fn a_project_category_overlapping_yours_is_a_heads_up_naming_both() {
    let files = Files::new(
        "return { category = { kind = { values = { 'bug' } }, place = { values = { 'home' } } } }",
        "return {}",
    );
    let store = files.store_path();
    let launcher = super::Helpers::default();
    let categories = Categories::new(vec![Category {
        name: "triage".into(),
        values: vec!["bug".into(), "feature".into()],
        exclusive: true,
    }])
    .unwrap();
    let inside = Standing::Inside(Box::new(Project {
        folder: "/work/app".into(),
        settings: ProjectSettings {
            categories,
            ..ProjectSettings::default()
        },
    }));
    let machine = Machine::new(&files.config, &store, &launcher, &NoProgress).within(inside);
    let found = serde_json::to_value(check(&machine)).unwrap();
    assert_eq!(found.as_array().unwrap().len(), 2, "{found}");
    assert_eq!(found[1]["name"], "config");
    assert_eq!(found[1]["result"], "warning");
    assert_eq!(
        found[1]["detail"],
        "your category kind is set aside in this project; the project's category triage takes its place"
    );
}
