mod as_one;
mod hooks;
mod preview;
mod selection;
mod tokens;
mod verbs;

use dam_domain::{Object, Oid};

use super::{Selector, SetChanges, SetRun};
use crate::config::{Config, FilterConfig, FilterRule};
use crate::testing::{Scenario, config, oid, task_in};
use crate::use_cases::edit::EditFields;

fn work_and_home() -> Scenario {
    let s = Scenario::default().configured(Config {
        filters: vec![FilterConfig {
            name: "home".into(),
            rule: FilterRule::Query("path:home/".into()),
        }],
        ..config()
    });
    s.working(Object::Task(task_in(1, "report", "work")));
    s.working(Object::Task(task_in(2, "invoice", "work")));
    s.working(Object::Task(task_in(3, "laundry", "home")));
    s
}

fn oids(bytes: &[u8]) -> Vec<Oid> {
    bytes.iter().map(|&b| oid(b)).collect()
}

fn named(bytes: &[u8]) -> Selector {
    Selector::Oids(oids(bytes))
}

fn query(text: &str) -> Selector {
    Selector::Query(text.into())
}

fn retitled(subject: &str) -> EditFields {
    EditFields {
        subject: Some(subject.into()),
        ..EditFields::default()
    }
}

fn run() -> SetRun {
    SetRun::default()
}

fn changed(changes: &SetChanges) -> Vec<Oid> {
    changes
        .changes
        .iter()
        .map(|c| c.change.oid().clone())
        .collect()
}

fn subject_of(s: &Scenario, byte: u8) -> String {
    use crate::testing::prelude::ObjectRepository;
    s.store
        .get(&oid(byte))
        .unwrap()
        .unwrap()
        .base()
        .subject
        .clone()
}
