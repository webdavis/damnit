use dam_adapters::LuaConfigFile;
use dam_application::{ConfigError, Project, ProjectSettings};

use dam_application::testing::FakeProjectHookSwitches;

use super::listing;
use crate::project_scope::Standing;

fn no_config() -> (tempfile::TempDir, LuaConfigFile) {
    let root = tempfile::tempdir().unwrap();
    let file = LuaConfigFile::new(root.path().join("config.lua"));
    (root, file)
}

#[test]
fn inside_a_project_the_list_shows_its_file_and_path_marked_as_the_projects() {
    let (_root, file) = no_config();
    let inside = Standing::Inside(Box::new(Project {
        folder: "/work/damnit".into(),
        settings: ProjectSettings {
            path: "damnit/".into(),
            ..Default::default()
        },
    }));
    let listed = listing::run(&file, &inside, &FakeProjectHookSwitches::default()).unwrap();
    assert_eq!(listed.data["project"]["file"], "/work/damnit/.dam.lua");
    assert_eq!(listed.data["project"]["path"], "damnit/");
    assert!(
        listed.human.contains("/work/damnit/.dam.lua"),
        "{}",
        listed.human
    );
    assert!(
        listed.human.contains("damnit/ (from the project)"),
        "{}",
        listed.human
    );
}

#[test]
fn outside_a_project_the_list_has_no_project() {
    let (_root, file) = no_config();
    let listed = listing::run(
        &file,
        &Standing::Outside,
        &FakeProjectHookSwitches::default(),
    )
    .unwrap();
    assert!(listed.data["project"].is_null());
    assert!(
        !listed.human.contains("from the project"),
        "{}",
        listed.human
    );
}

#[test]
fn an_unreadable_project_file_is_a_warning_and_the_list_carries_on() {
    let (_root, file) = no_config();
    let unreadable = Standing::Unreadable(ConfigError::NotATable {
        path: "/work/damnit/.dam.lua".into(),
        returned: "a number".into(),
    });
    let warning = unreadable.warning().unwrap();
    let listed = listing::run(&file, &unreadable, &FakeProjectHookSwitches::default()).unwrap();
    assert!(listed.data["project"].is_null());
    assert_eq!(listed.data["warnings"][0], warning.as_str());
}

#[test]
fn a_project_file_holding_an_escape_sequence_lists_no_control_character() {
    let (root, file) = no_config();
    let project = root.path().join("app");
    std::fs::create_dir_all(&project).unwrap();
    let text = "return { path = \"\\27]0;evil\\7\" }";
    std::fs::write(project.join(".dam.lua"), text).unwrap();
    let standing = Standing::of(&project, &root.path().join("config"));
    let listed = listing::run(&file, &standing, &FakeProjectHookSwitches::default()).unwrap();
    let printed = format!("{}{}", listed.human.replace('\n', " "), listed.data);
    assert!(!printed.chars().any(char::is_control), "{printed:?}");
    assert!(!printed.contains("evil"), "{printed}");
}

#[test]
fn a_remote_lists_the_paths_it_never_receives() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config.lua");
    std::fs::write(
        &path,
        "return { remote = { todoist = { url = 'todoist::', except = { 'kitchen', 'journal/private' } } } }",
    )
    .unwrap();
    let none = FakeProjectHookSwitches::default();
    let listed = listing::run(&LuaConfigFile::new(path), &Standing::Outside, &none).unwrap();
    assert!(
        listed.human.contains("remote.todoist.except") && listed.human.contains("journal/private"),
        "{}",
        listed.human
    );
}

#[test]
fn inside_a_project_the_list_shows_every_project_setting_marked_as_the_projects() {
    let (_root, file) = no_config();
    let inside = Standing::Inside(Box::new(Project {
        folder: "/work/damnit".into(),
        settings: ProjectSettings {
            path: "damnit/".into(),
            minimum_dam: Some("0.2.0".into()),
            new_task: dam_application::TaskTemplate {
                labels: Some(vec!["cli".into()]),
                priority: Some(dam_domain::Priority::new(2).unwrap()),
                due: Some("friday".into()),
                ..Default::default()
            },
            required: vec![dam_application::RequiredField::Due],
            ls: Some("open".into()),
            filters: vec![dam_application::FilterConfig {
                name: "open".into(),
                rule: dam_application::FilterRule::Query("label:cli".into()),
            }],
            categories: dam_domain::Categories::new(vec![dam_domain::Category {
                name: "area".into(),
                values: vec!["ui".into(), "db".into()],
                exclusive: true,
            }])
            .unwrap(),
            sections: vec!["bugs".into(), "features".into()],
            branch_label: true,
            done_children: Some(dam_domain::ChildDisposition::Into("rest".into())),
        },
    }));
    let listed = listing::run(&file, &inside, &FakeProjectHookSwitches::default()).unwrap();
    let project = &listed.data["project"];
    assert_eq!(project["minimum_dam"], "0.2.0");
    assert_eq!(project["new_task"]["labels"][0], "cli");
    assert_eq!(project["new_task"]["priority"], 2);
    assert_eq!(project["new_task"]["due"], "friday");
    assert!(project["new_task"]["deadline"].is_null());
    assert_eq!(project["required"][0], "due");
    assert_eq!(project["ls"], "open");
    assert_eq!(project["filter"]["open"]["query"], "label:cli");
    assert_eq!(project["category"]["area"]["values"][1], "db");
    assert_eq!(project["category"]["area"]["exclusive"], true);
    assert_eq!(project["sections"][1], "features");
    assert_eq!(project["branch_label"], true);
    assert_eq!(project["done_children"], "into:rest");
    for row in [
        "0.2.0 (from the project)",
        "into:rest (from the project)",
        "cli (from the project)",
        "friday (from the project)",
        "due (from the project)",
        "open (from the project)",
        "label:cli (from the project)",
        "ui, db (from the project)",
        "true (from the project)",
        "bugs, features (from the project)",
    ] {
        assert!(listed.human.contains(row), "{row}: {}", listed.human);
    }
}
