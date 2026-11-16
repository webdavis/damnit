use dam_application::{ConfigError, Misread, TemplateOwner};
use dam_domain::{Path, Priority};

use super::template_support::{Folders, found, names};

const EVERY_FIELD: &str = r#"
return {
  body = "Steps to reproduce:",
  labels = { "bug", "triage" },
  priority = 2,
  due = "tomorrow",
  deadline = "fri",
  path = "bugs/",
}
"#;

#[test]
fn a_template_holds_each_field_it_sets() {
    let folders = Folders::new().own_template("bug", EVERY_FIELD);
    let template = found(&folders.outside(), "bug").unwrap().unwrap();
    assert_eq!(template.body.as_deref(), Some("Steps to reproduce:"));
    assert_eq!(template.labels, Some(vec!["bug".into(), "triage".into()]));
    assert_eq!(template.priority, Some(Priority::new(2).unwrap()));
    assert_eq!(template.due.as_deref(), Some("tomorrow"));
    assert_eq!(template.deadline.as_deref(), Some("fri"));
    assert_eq!(template.path, Some(Path::parse("bugs/").unwrap()));
}

#[test]
fn a_persons_template_may_compute_a_value() {
    let folders = Folders::new().own_template("chore", "return { body = string.rep('x', 3) }");
    let template = found(&folders.outside(), "chore").unwrap().unwrap();
    assert_eq!(template.body.as_deref(), Some("xxx"));
}

#[test]
fn an_unknown_key_is_refused_by_name() {
    let folders = Folders::new().own_template("bug", "return { subject = 'x' }");
    match found(&folders.outside(), "bug").unwrap_err() {
        ConfigError::Misread(misread) => {
            assert!(matches!(*misread, Misread::UnknownKey { ref key, .. } if key == "subject"))
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_priority_outside_one_to_four_is_refused() {
    let folders = Folders::new().own_template("bug", "return { priority = 9 }");
    assert!(found(&folders.outside(), "bug").is_err());
}

#[test]
fn the_projects_template_comes_before_the_persons() {
    let folders = Folders::new()
        .project_template("bug", "return { body = 'project' }")
        .own_template("bug", "return { body = 'own' }");
    let template = found(&folders.inside(), "bug").unwrap().unwrap();
    assert_eq!(template.body.as_deref(), Some("project"));
}

#[test]
fn outside_a_project_only_the_persons_templates_are_read() {
    let folders = Folders::new()
        .project_template("bug", "return { body = 'project' }")
        .own_template("chore", "return {}");
    assert_eq!(found(&folders.outside(), "bug").unwrap(), None);
    assert_eq!(
        names(&folders.outside()),
        vec![("chore".into(), TemplateOwner::Person)]
    );
}

#[test]
fn a_name_found_nowhere_is_none_and_names_lists_both_folders() {
    let folders = Folders::new()
        .project_template("bug", "return {}")
        .own_template("chore", "return {}");
    assert_eq!(found(&folders.inside(), "missing").unwrap(), None);
    assert_eq!(
        names(&folders.inside()),
        vec![
            ("bug".into(), TemplateOwner::Project),
            ("chore".into(), TemplateOwner::Person)
        ]
    );
}

#[test]
fn no_folder_lists_no_names() {
    assert!(names(&Folders::new().inside()).is_empty());
}
