mod map;
mod places;
mod run;

use dam_domain::{Categories, Category, FilterTable, Object, TableValue, Task};

use super::LabelMap;
use crate::config::{Config, FilterConfig};
use crate::testing::{Scenario, config, oid};

fn effort() -> Categories {
    Categories::new(vec![
        Category {
            name: "effort".into(),
            values: vec!["quick".into(), "hard".into(), "deep".into()],
            exclusive: true,
        },
        Category {
            name: "place".into(),
            values: vec!["home".into(), "work".into()],
            exclusive: false,
        },
    ])
    .unwrap()
}

fn table(fields: &[(&str, &str)], any: Vec<FilterTable>) -> FilterTable {
    FilterTable {
        fields: fields
            .iter()
            .map(|(k, v)| (k.to_string(), TableValue::One(v.to_string())))
            .collect(),
        any,
    }
}

fn filter(name: &str, table: FilterTable, runs_a_function: bool) -> FilterConfig {
    FilterConfig {
        name: name.into(),
        table,
        runs_a_function,
    }
}

fn short(label: &str) -> FilterTable {
    table(
        &[],
        vec![
            table(&[("labels", label)], vec![]),
            table(&[("due", "today")], vec![]),
        ],
    )
}

fn configured() -> Config {
    Config {
        categories: effort(),
        filters: vec![
            filter("short", short("quick"), false),
            filter(
                "focus",
                table(&[("category", "effort=hard")], vec![]),
                false,
            ),
            filter("home", table(&[("labels", "home")], vec![]), false),
            filter("picked", table(&[("labels", "quick")], vec![]), true),
            filter("coded", FilterTable::default(), true),
        ],
        ..config()
    }
}

fn map(pairs: &[(&str, &str)]) -> LabelMap {
    LabelMap::new(
        pairs
            .iter()
            .map(|(old, new)| (old.to_string(), new.to_string()))
            .collect(),
        &effort(),
    )
    .unwrap()
}

fn labelled(byte: u8, labels: &[&str]) -> Object {
    let mut task = Task::new(oid(byte), format!("t{byte}"));
    task.base.labels = labels.iter().map(|l| l.to_string()).collect();
    Object::Task(task)
}

fn scenario(objects: Vec<Object>) -> Scenario {
    let s = Scenario::default().configured(configured());
    for object in objects {
        s.working(object);
    }
    s
}
