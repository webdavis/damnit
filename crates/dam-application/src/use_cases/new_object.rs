use std::collections::BTreeSet;

use dam_domain::{Date, Event, Object, Oid, Path, Priority, Task, When};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;

pub struct NewTask {
    pub subject: String,
    pub path: Path,
    pub priority: Priority,
    pub due: Option<When>,
    pub deadline: Option<Date>,
    pub labels: BTreeSet<String>,
    pub body: String,
}

pub struct NewEvent {
    pub subject: String,
    pub path: Path,
    pub start: When,
    pub end: When,
    pub labels: BTreeSet<String>,
    pub body: String,
}

impl Dam<'_> {
    pub fn new_task(&self, input: NewTask) -> Result<Oid, UseCaseError> {
        atomically(self.store, || {
            self.config
                .categories
                .check(&input.labels)
                .map_err(Refusal::Labels)?;
            let oid = Oid::generate(&mut |b| self.random.fill(b));
            let mut task = Task::new(oid.clone(), input.subject);
            task.base.path = input.path;
            task.base.labels = input.labels;
            task.base.body = input.body;
            task.priority = input.priority;
            task.due = input.due;
            task.deadline = input.deadline;
            self.store.put(&Object::Task(task))?;
            Ok(oid)
        })
    }

    pub fn new_event(&self, input: NewEvent) -> Result<Oid, UseCaseError> {
        atomically(self.store, || {
            self.config
                .categories
                .check(&input.labels)
                .map_err(Refusal::Labels)?;
            let oid = Oid::generate(&mut |b| self.random.fill(b));
            let mut event = Event::new(oid.clone(), input.subject, input.start, input.end);
            event.base.path = input.path;
            event.base.labels = input.labels;
            event.base.body = input.body;
            self.store.put(&Object::Event(event))?;
            Ok(oid)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::testing::prelude::*;
    use crate::testing::{FaultyStore, FixedRandom, Fixture, MemoryStore, Scenario, oid};
    use dam_domain::{Categories, Category};

    fn input(subject: &str, labels: &[&str]) -> NewTask {
        NewTask {
            subject: subject.into(),
            path: Path::root(),
            priority: Priority::default(),
            due: None,
            deadline: None,
            labels: labels.iter().map(|s| s.to_string()).collect(),
            body: String::new(),
        }
    }

    fn effort() -> Categories {
        Categories::new(vec![Category {
            name: "effort".into(),
            values: vec!["light".into(), "deep".into()],
            exclusive: true,
        }])
        .unwrap()
    }

    fn dentist(start: When, end: When) -> NewEvent {
        NewEvent {
            subject: "dentist".into(),
            path: Path::root(),
            start,
            end,
            labels: Default::default(),
            body: String::new(),
        }
    }

    fn drawing_from(byte: u8) -> Fixture {
        Fixture {
            random: FixedRandom::new(byte),
            ..Fixture::default()
        }
    }

    #[test]
    fn a_new_task_lands_in_working_with_a_generated_oid_and_nothing_staged() {
        let s = Scenario::default().drawing_from(7);
        let id = s.dam().new_task(input("milk", &[])).unwrap();
        assert_eq!(id, oid(7));
        let got = s.store.get(&id).unwrap().unwrap();
        assert_eq!(got.base().subject, "milk");
        assert!(s.store.staged().unwrap().is_empty());
        assert!(s.store.committed(&id).unwrap().is_none());
    }

    #[test]
    fn labels_are_checked_against_categories() {
        let s = Scenario::default().drawing_from(1).configured(Config {
            categories: effort(),
            ..Config::default()
        });
        let err = s
            .dam()
            .new_task(input("x", &["light", "deep"]))
            .unwrap_err();
        assert!(matches!(err, UseCaseError::Refused(Refusal::Labels(_))));
        assert!(s.store.all().unwrap().is_empty());
    }

    #[test]
    fn a_new_event_lands_in_working() {
        let s = Scenario::default().drawing_from(2);
        let start = When::Day(jiff::civil::date(2026, 9, 25));
        let end = When::Day(jiff::civil::date(2026, 9, 26));
        let id = s.dam().new_event(dentist(start, end)).unwrap();
        assert!(matches!(s.store.get(&id).unwrap(), Some(Object::Event(_))));
    }

    #[test]
    fn a_new_task_that_fails_at_its_write_leaves_nothing_behind() {
        let store = FaultyStore::new(MemoryStore::new());
        let fixture = drawing_from(7);
        let dam = fixture.dam(&store);
        store.assert_unchanged_when_any_write_fails(|| dam.new_task(input("milk", &[])).map(drop));
    }

    #[test]
    fn a_new_event_that_fails_at_its_write_leaves_nothing_behind() {
        let store = FaultyStore::new(MemoryStore::new());
        let day = When::Day(jiff::civil::date(2026, 9, 25));
        let fixture = drawing_from(2);
        let dam = fixture.dam(&store);
        store.assert_unchanged_when_any_write_fails(|| {
            dam.new_event(dentist(day.clone(), day.clone())).map(drop)
        });
    }
}
