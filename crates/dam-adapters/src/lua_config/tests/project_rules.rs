use dam_application::{ConfigError, FilterConfig, ProjectRuleFault, RequiredField, TaskTemplate};
use dam_domain::{FilterTable, Priority, TableValue};

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

fn priority_one() -> FilterTable {
    FilterTable {
        fields: [("priority".into(), TableValue::One("1".into()))].into(),
        any: Vec::new(),
    }
}

#[test]
fn the_default_list_is_read_as_a_table() {
    let folder = ProjectFolder::with("return { path = 'a/', list = { priority = 1 } }");
    assert_eq!(folder.read().unwrap().list, Some(priority_one()));
    refused_as_misshapen("return { path = 'a/', list = 3 }", "list");
}

#[test]
fn the_old_ls_is_refused_naming_list_with_its_table() {
    let folder = ProjectFolder::with("return { path = 'a/', ls = '@cli & p1' }");
    assert_eq!(
        folder.read(),
        Err(ConfigError::QueryText {
            file: folder.file().display().to_string(),
            was: "ls".into(),
            key: "list".into(),
            table: Some(r#"{ done = "any", labels = "cli", priority = "1" }"#.into()),
        })
    );
}

#[test]
fn the_projects_filters_are_read_as_tables() {
    let folder =
        ProjectFolder::with("return { path = 'a/', filter = { mine = { priority = 1 } } }");
    assert_eq!(
        folder.read().unwrap().filters,
        [FilterConfig {
            name: "mine".into(),
            table: priority_one(),
            runs_a_function: false,
        }]
    );
}

#[test]
fn a_projects_filter_in_the_old_form_is_refused_with_its_table() {
    let folder =
        ProjectFolder::with("return { path = 'a/', filter = { mine = { query = 'p1' } } }");
    let Err(ConfigError::QueryText { was, table, .. }) = folder.read() else {
        panic!("the old form was read");
    };
    assert_eq!(was, "filter.mine.query");
    assert_eq!(
        table.as_deref(),
        Some(r#"{ done = "any", priority = "1" }"#)
    );
}

#[test]
fn a_projects_filter_that_reaches_itself_is_refused() {
    let folder = ProjectFolder::with(
        "return { path = 'a/', filter = { a = { filter = 'b' }, b = { filter = 'a' } } }",
    );
    assert!(matches!(
        folder.read(),
        Err(ConfigError::FilterReachesItself { .. })
    ));
}

#[test]
fn a_filter_that_holds_a_function_is_refused_as_not_a_query() {
    let folder = ProjectFolder::with(
        "return { path = 'a/', filter = { mine = { match = function() return true end } } }",
    );
    assert_eq!(
        folder.read(),
        Err(ConfigError::ProjectRule {
            file: folder.file(),
            key: "filter",
            fault: ProjectRuleFault::FilterIsAFunction,
        })
    );
    refused_as_misshapen("return { path = 'a/', filter = { mine = 'p1' } }", "filter");
}
