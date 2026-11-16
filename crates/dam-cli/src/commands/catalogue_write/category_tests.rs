use dam_application::testing::{FakeSettingsFile, MemoryStore, Written, oid};
use dam_application::{Config, ObjectRepository, SettingValue};
use dam_domain::{Categories, Category, Event, Object, Task, When};

use super::category::{create, remove};
use crate::error::CliError;
use crate::output::Report;

fn effort() -> Config {
    Config {
        categories: Categories::new(vec![Category {
            name: "effort".into(),
            values: vec!["light".into(), "deep".into()],
            exclusive: true,
        }])
        .unwrap(),
        ..Config::default()
    }
}

fn refusal(result: Result<Report, CliError>) -> String {
    match result {
        Err(error) => error.to_string(),
        Ok(_) => panic!("was not refused"),
    }
}

fn list(items: &[&str]) -> SettingValue {
    SettingValue::List(items.iter().map(|item| item.to_string()).collect())
}

#[test]
fn creating_an_exclusive_category_writes_its_values_and_the_flag() {
    let file = FakeSettingsFile::default();
    let report = create(&Config::default(), &file, "effort", "light, deep", true).unwrap();
    assert_eq!(
        *file.written.borrow(),
        vec![
            Written::Set("category.effort.values".into(), list(&["light", "deep"])),
            Written::Set("category.effort.exclusive".into(), SettingValue::Flag(true)),
        ]
    );
    assert_eq!(
        report.human,
        "created category effort: light, deep (exclusive)"
    );
    assert_eq!(
        report.data,
        serde_json::json!({"name": "effort", "values": ["light", "deep"], "exclusive": true})
    );
}

#[test]
fn a_category_that_is_not_exclusive_writes_only_its_values() {
    let file = FakeSettingsFile::default();
    let report = create(&Config::default(), &file, "place", "home,work", false).unwrap();
    assert_eq!(
        *file.written.borrow(),
        vec![Written::Set(
            "category.place.values".into(),
            list(&["home", "work"])
        )]
    );
    assert_eq!(report.human, "created category place: home, work");
}

#[test]
fn the_rules_of_the_config_reader_refuse_a_category_and_nothing_is_written() {
    let file = FakeSettingsFile::default();
    let cases = [
        (
            "effort",
            "quick",
            "category effort exists; change its values with dam config set category.effort.values '<values>'",
        ),
        (
            "focus",
            "deep",
            "deep already belongs to category effort; a label belongs to one category, and dam category list shows each one's labels",
        ),
        ("place", "home,home", "--values names home twice"),
        (
            "effort",
            "",
            "category effort exists; change its values with dam config set category.effort.values '<values>'",
        ),
        (
            "place",
            " , ",
            "category place needs at least one label in --values",
        ),
        ("", "home", "a category needs a name"),
    ];
    for (name, values, refused) in cases {
        assert_eq!(
            refusal(create(&effort(), &file, name, values, false)),
            refused
        );
    }
    assert!(file.written.borrow().is_empty());
}

#[test]
fn removing_a_category_removes_its_entry_and_counts_the_tasks_keeping_its_labels() {
    let store = MemoryStore::new();
    for (at, labels) in [
        (1, vec!["light"]),
        (2, vec!["deep", "home"]),
        (3, vec!["home"]),
    ] {
        let mut task = Task::new(oid(at), "a task");
        task.base.labels = labels.into_iter().map(str::to_string).collect();
        store.put(&Object::Task(task)).unwrap();
    }
    let day = When::Day(jiff::civil::date(2026, 9, 25));
    let mut event = Event::new(oid(4), "an event", day.clone(), day);
    event.base.labels = ["light".to_string()].into();
    store.put(&Object::Event(event)).unwrap();
    let file = FakeSettingsFile::default();
    let report = remove(&effort(), &store, &file, "effort").unwrap();
    assert_eq!(
        *file.written.borrow(),
        vec![Written::Unset("category.effort".into())]
    );
    assert_eq!(
        report.human,
        "removed category effort\n2 tasks keep its labels as free labels"
    );
    assert_eq!(
        report.data,
        serde_json::json!({"name": "effort", "tasks_keeping_labels": 2})
    );
}

#[test]
fn removing_a_category_no_task_carries_says_only_that_it_is_removed() {
    let file = FakeSettingsFile::default();
    let report = remove(&effort(), &MemoryStore::new(), &file, "effort").unwrap();
    assert_eq!(report.human, "removed category effort");
}

#[test]
fn removing_a_category_that_does_not_exist_is_refused() {
    let file = FakeSettingsFile::default();
    assert_eq!(
        refusal(remove(&effort(), &MemoryStore::new(), &file, "place")),
        "there is no category place; dam category list shows the categories"
    );
    assert!(file.written.borrow().is_empty());
}

#[test]
fn the_answers_match_their_schemas() {
    let file = FakeSettingsFile::default();
    let created = create(&Config::default(), &file, "effort", "light", true).unwrap();
    let removed = remove(&effort(), &MemoryStore::new(), &file, "effort").unwrap();
    for (data, schema) in [
        (
            created.data,
            include_str!("../../../../../docs/schemas/category-create.schema.json"),
        ),
        (
            removed.data,
            include_str!("../../../../../docs/schemas/category-remove.schema.json"),
        ),
    ] {
        let schema: serde_json::Value = serde_json::from_str(schema).unwrap();
        assert!(jsonschema::is_valid(&schema, &data), "{data}");
    }
}

#[test]
fn one_task_keeping_the_labels_is_counted_in_the_singular() {
    let store = MemoryStore::new();
    let mut task = Task::new(oid(1), "a task");
    task.base.labels = ["deep".to_string()].into();
    store.put(&Object::Task(task)).unwrap();
    let report = remove(&effort(), &store, &FakeSettingsFile::default(), "effort").unwrap();
    assert_eq!(
        report.human,
        "removed category effort\n1 task keeps its labels as free labels"
    );
}
