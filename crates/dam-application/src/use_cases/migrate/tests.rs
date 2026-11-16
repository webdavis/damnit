mod map;
mod places;
mod run;

use dam_domain::{Categories, Category, Object, Task};

use super::LabelMap;
use crate::config::{Config, FilterConfig, FilterRule};
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

fn filter(name: &str, rule: FilterRule) -> FilterConfig {
    FilterConfig {
        name: name.into(),
        rule,
    }
}

fn configured() -> Config {
    Config {
        categories: effort(),
        filters: vec![
            filter("short", FilterRule::Query("@quick | due:today".into())),
            filter("focus", FilterRule::Query("effort:hard & !done".into())),
            filter("home", FilterRule::Query("@home".into())),
            filter("picked", FilterRule::QueryThenFunction("@quick".into())),
            filter("coded", FilterRule::Function),
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
