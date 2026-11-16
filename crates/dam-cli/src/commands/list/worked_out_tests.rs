use crate::testing::{context, list_args};
use dam_domain::{Categories, Category, Object, Oid, Task, When};
use jiff::civil::date;
use serde_json::{Value, json};

const LIST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/list.schema.json"
));
const OBJECT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/object.schema.json"
));

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn rows(words: &[&str]) -> Vec<Value> {
    let mut ctx = context();
    ctx.config.categories = Categories::new(vec![
        Category {
            name: "effort".into(),
            values: vec!["deep".into(), "quick".into()],
            exclusive: true,
        },
        Category {
            name: "area".into(),
            values: vec!["ui".into(), "db".into()],
            exclusive: false,
        },
    ])
    .unwrap();
    let mut late = Task::new(oid(1), "late");
    late.due = Some(When::Day(date(2026, 9, 1)));
    late.base.depends = vec![oid(2)];
    late.base.labels = ["deep", "ui", "db", "home"].map(String::from).into();
    ctx.store.put(&Object::Task(late)).unwrap();
    ctx.store
        .put(&Object::Task(Task::new(oid(2), "first")))
        .unwrap();
    let report = super::run_list(&ctx, list_args(words)).unwrap();
    report.data["objects"].as_array().unwrap().clone()
}

#[test]
fn a_row_carries_its_category_overdue_and_blocked() {
    let rows = rows(&["--sort", "subject"]);
    let late = &rows[1];
    assert_eq!(late["subject"], "late");
    assert_eq!(
        late["category"],
        json!({"effort": "deep", "area": ["db", "ui"]})
    );
    assert_eq!(late["overdue"], true);
    assert_eq!(late["blocked"], true);
    let first = &rows[0];
    assert_eq!(first["category"], json!({}));
    assert_eq!(first["overdue"], false);
    assert_eq!(first["blocked"], false);
}

#[test]
fn the_printed_keys_agree_with_what_the_flags_select() {
    for (flag, key) in [("--overdue", "overdue"), ("--blocked", "blocked")] {
        let selected = rows(&[flag]);
        assert!(!selected.is_empty());
        assert!(selected.iter().all(|row| row[key] == true), "{flag}");
    }
    let deep = rows(&["--category", "effort=deep"]);
    assert!(deep.iter().all(|row| row["category"]["effort"] == "deep"));
}

#[test]
fn the_rows_keep_the_list_schema() {
    let document = json!({"objects": rows(&["--done", "any"])});
    let object: Value = serde_json::from_str(OBJECT).unwrap();
    let id = object["$id"].as_str().unwrap().to_string();
    let validator = jsonschema::options()
        .with_resource(id, jsonschema::Resource::from_contents(object).unwrap())
        .build(&serde_json::from_str(LIST).unwrap())
        .unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&document)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{document}: {errors:?}");
}
