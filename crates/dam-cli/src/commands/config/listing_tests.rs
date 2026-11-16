use dam_adapters::LuaConfigFile;
use dam_application::{ConfigError, Project, ProjectSettings};

use super::listing::run;
use crate::project_scope::Standing;

fn no_config() -> (tempfile::TempDir, LuaConfigFile) {
    let root = tempfile::tempdir().unwrap();
    let file = LuaConfigFile::new(root.path().join("config.lua"));
    (root, file)
}

#[test]
fn inside_a_project_the_list_shows_its_file_and_path_marked_as_the_projects() {
    let (_root, file) = no_config();
    let inside = Standing::Inside(Project {
        folder: "/work/damnit".into(),
        settings: ProjectSettings {
            path: "damnit/".into(),
            ..Default::default()
        },
    });
    let listed = run(&file, &inside).unwrap();
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
    let listed = run(&file, &Standing::Outside).unwrap();
    assert!(listed.data["project"].is_null());
    assert!(!listed.human.contains("project"), "{}", listed.human);
}

#[test]
fn an_unreadable_project_file_is_a_warning_and_the_list_carries_on() {
    let (_root, file) = no_config();
    let unreadable = Standing::Unreadable(ConfigError::NotATable {
        path: "/work/damnit/.dam.lua".into(),
        returned: "a number".into(),
    });
    let warning = unreadable.warning().unwrap();
    let listed = run(&file, &unreadable).unwrap();
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
    let listed = run(&file, &standing).unwrap();
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
    let listed = run(&LuaConfigFile::new(path), &Standing::Outside).unwrap();
    assert!(
        listed.human.contains("remote.todoist.except") && listed.human.contains("journal/private"),
        "{}",
        listed.human
    );
}
