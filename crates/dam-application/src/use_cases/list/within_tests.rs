use dam_domain::Permission::TaskRead;
use dam_domain::{Kind, Object, Path};

use crate::config::Config;
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, task_in};

fn a_project_and_more() -> Scenario {
    let s = Scenario::default().configured(Config {
        filters: vec![
            crate::testing::saved("cli", &[("path", "damnit/cli/")]),
            crate::testing::saved("everything", &[("kind", "task")]),
        ],
        ..Config::default()
    });
    s.working(Object::Task(task_in(1, "damnit", "")));
    s.working(Object::Task(task_in(2, "parse flags", "damnit/cli")));
    s.working(Object::Task(task_in(3, "readme", "damnit")));
    s.working(Object::Task(task_in(4, "laundry", "home")));
    s.working(Object::Task(task_in(5, "other cli", "cli")));
    s
}

fn damnit() -> Path {
    Path::parse("damnit/").unwrap()
}

fn subjects(objects: Vec<Object>) -> Vec<String> {
    objects
        .into_iter()
        .map(|o| o.base().subject.clone())
        .collect()
}

#[test]
fn a_list_within_a_project_holds_what_lies_under_it() {
    let s = a_project_and_more();
    assert_eq!(
        subjects(s.dam().list_within(&damnit()).unwrap()),
        ["readme", "parse flags"]
    );
}

#[test]
fn a_saved_filter_keeps_its_paths_from_the_root_and_is_narrowed_to_the_project() {
    let s = a_project_and_more();
    let dam = s.dam();
    let within = |name: &str| {
        let listed = dam.listed_by_name(name, &damnit(), &[]).unwrap();
        subjects(listed)
    };
    assert_eq!(within("cli"), ["parse flags"]);
    assert_eq!(within("everything"), ["readme", "parse flags"]);
}

#[test]
fn a_token_lists_only_what_it_sees_inside_the_project() {
    let s = a_project_and_more();
    let listed = |paths: Option<&[&str]>| {
        let token = holder(vec![allowed(&[TaskRead], paths)]);
        let dam = Dam {
            access: &token,
            ..s.dam()
        };
        subjects(dam.list_within(&damnit()).unwrap())
    };
    assert!(listed(Some(&["home"])).is_empty());
    assert_eq!(listed(Some(&["damnit/cli"])), ["parse flags"]);
    assert_eq!(listed(None), ["readme", "parse flags"]);
}

#[test]
fn only_the_kinds_task_and_event_exist() {
    assert_eq!(Kind::parse("task"), Some(Kind::Task));
    assert_eq!(Kind::parse("event"), Some(Kind::Event));
    assert_eq!(Kind::parse("project"), None);
}

#[test]
fn the_projects_own_task_can_still_be_edited_dated_and_completed() {
    use crate::ports::ObjectRepository;
    use crate::use_cases::edit::EditFields;
    use dam_domain::Force;
    use dam_domain::{Tracked, When};
    let s = Scenario::default();
    s.working(Object::Task(task_in(1, "damnit", "")));
    let dam = s.dam();
    let project = crate::testing::oid(1);
    let due = When::Day(jiff::civil::date(2026, 9, 20));
    let fields = EditFields {
        subject: Some("damnit tasks".into()),
        due: Some(Some(due.clone())),
        ..EditFields::default()
    };
    dam.edit(&project, fields).unwrap();
    dam.complete(&project, Force::No, Tracked::No).unwrap();
    let Some(Object::Task(task)) = s.store.get(&project).unwrap() else {
        panic!("the project task is gone")
    };
    assert_eq!(task.base.subject, "damnit tasks");
    assert_eq!(task.due, Some(due));
    assert!(task.completion.is_done());
}
