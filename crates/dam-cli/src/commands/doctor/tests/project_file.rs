use dam_application::ConfigError;

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
