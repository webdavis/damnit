use dam_application::{ConfigError, ProjectRuleFault, RequiredField, TaskTemplate};
use dam_domain::Priority;

use super::project_support::ProjectFolder;

fn refused_as_misshapen(text: &str, key: &'static str) {
    let folder = ProjectFolder::with(text);
    assert_eq!(
        folder.read(),
        Err(ConfigError::ProjectRule {
            file: folder.file(),
            key,
            fault: ProjectRuleFault::Shape,
        }),
        "{text}"
    );
}

#[test]
fn defaults_for_a_new_task_are_read() {
    let folder = ProjectFolder::with(
        "return { path = 'a/', new_task = { labels = { 'cli' }, priority = 2, due = 'tomorrow', \
         deadline = 'friday' } }",
    );
    assert_eq!(
        folder.read().unwrap().new_task,
        TaskTemplate {
            labels: Some(vec!["cli".into()]),
            priority: Some(Priority::new(2).unwrap()),
            due: Some("tomorrow".into()),
            deadline: Some("friday".into()),
            ..TaskTemplate::default()
        }
    );
}

#[test]
fn defaults_of_the_wrong_shape_are_refused_naming_only_the_file() {
    refused_as_misshapen("return { path = 'a/', new_task = 'x' }", "new_task");
    refused_as_misshapen(
        "return { path = 'a/', new_task = { body = 'x' } }",
        "new_task",
    );
    refused_as_misshapen(
        "return { path = 'a/', new_task = { priority = 9 } }",
        "new_task",
    );
    refused_as_misshapen(
        "return { path = 'a/', new_task = { labels = 'x' } }",
        "new_task",
    );
}

#[test]
fn the_required_fields_are_read() {
    let folder = ProjectFolder::with(
        "return { path = 'a/', required = { 'due', 'labels', 'priority', 'body', 'deadline' } }",
    );
    assert_eq!(
        folder.read().unwrap().required,
        [
            RequiredField::Due,
            RequiredField::Labels,
            RequiredField::Priority,
            RequiredField::Body,
            RequiredField::Deadline,
        ]
    );
}

#[test]
fn a_required_field_dam_does_not_know_is_refused_naming_only_the_file() {
    refused_as_misshapen(
        "return { path = 'a/', required = { 'colour' } }",
        "required",
    );
    refused_as_misshapen("return { path = 'a/', required = 'due' }", "required");
}

#[test]
fn the_default_listing_is_read() {
    let folder = ProjectFolder::with("return { path = 'a/', ls = '@cli & p1' }");
    assert_eq!(folder.read().unwrap().ls.as_deref(), Some("@cli & p1"));
}

#[test]
fn a_default_listing_that_is_not_a_query_is_refused_naming_only_the_file() {
    let folder = ProjectFolder::with("return { path = 'a/', ls = '@cli &' }");
    assert_eq!(
        folder.read(),
        Err(ConfigError::ProjectRule {
            file: folder.file(),
            key: "ls",
            fault: ProjectRuleFault::NotAQuery,
        })
    );
    refused_as_misshapen("return { path = 'a/', ls = 3 }", "ls");
}
