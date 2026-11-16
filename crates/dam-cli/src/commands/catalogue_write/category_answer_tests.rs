use dam_application::testing::{FakeSettingsFile, MemoryStore, oid};
use dam_application::{Config, ObjectRepository};
use dam_domain::{Categories, Category, Object, Task};

use super::category::{create, remove};

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

#[test]
fn the_answers_match_their_schemas() {
    let file = FakeSettingsFile::default();
    let created = create(
        &Config::default(),
        &file,
        "config.lua",
        &Categories::default(),
        ("effort", "light", true),
    )
    .unwrap();
    let removed = remove(
        &effort(),
        &MemoryStore::new(),
        &file,
        "config.lua",
        "effort",
    )
    .unwrap();
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
    let report = remove(
        &effort(),
        &store,
        &FakeSettingsFile::default(),
        "config.lua",
        "effort",
    )
    .unwrap();
    assert_eq!(
        report.human,
        "removed category effort\n1 task keeps its labels as free labels"
    );
}
