use dam_application::{ConfigError, ProjectRuleFault};
use dam_domain::{Categories, Category};

use super::project_support::ProjectFolder;

fn refused(text: &str, fault: ProjectRuleFault) {
    let folder = ProjectFolder::with(text);
    let read = folder.read();
    assert_eq!(
        read,
        Err(ConfigError::ProjectRule {
            file: folder.file(),
            key: "category",
            fault,
        }),
        "{text}"
    );
}

#[test]
fn label_categories_are_read_as_config_lua_writes_them() {
    let folder = ProjectFolder::with(
        "return { path = 'a/', category = { kind = { values = { 'bug', 'feature' }, exclusive = true } } }",
    );
    assert_eq!(
        folder.read().unwrap().categories,
        Categories::new(vec![Category {
            name: "kind".into(),
            values: vec!["bug".into(), "feature".into()],
            exclusive: true,
        }])
        .unwrap()
    );
}

#[test]
fn a_category_of_the_wrong_shape_is_refused_naming_only_the_file() {
    for text in [
        "return { path = 'a/', category = 'kind' }",
        "return { path = 'a/', category = { kind = { values = 'bug' } } }",
        "return { path = 'a/', category = { kind = { values = { 'bug' }, colour = 1 } } }",
        "return { path = 'a/', category = { kind = { values = { 'bug' }, exclusive = 'yes' } } }",
        "return { path = 'a/', category = { kind = {} } }",
    ] {
        refused(text, ProjectRuleFault::Shape);
    }
}

#[test]
fn a_label_in_two_project_categories_is_refused_without_quoting_it() {
    refused(
        "return { path = 'a/', category = { kind = { values = { 'bug' } }, size = { values = { 'bug' } } } }",
        ProjectRuleFault::LabelInTwoCategories,
    );
    refused(
        "return { path = 'a/', category = { kind = { values = {} } } }",
        ProjectRuleFault::Shape,
    );
}

#[test]
fn a_table_may_name_the_projects_own_categories() {
    let folder = ProjectFolder::with(
        "return { path = 'a/', list = { category = 'area=bug' }, \
         filter = { bugs = { category = 'area=bug' } }, \
         category = { area = { values = { 'bug' } } } }",
    );
    let list = folder.read().unwrap().list.unwrap();
    assert_eq!(list.fields["category"].texts(), ["area=bug"]);
}
