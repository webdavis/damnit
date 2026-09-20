use std::collections::BTreeSet;

use dam_domain::{Categories, Date, Event, Object, Oid, Path, Priority, Task, When};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{ObjectRepository, Randomness};

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

pub fn new_task(
    objects: &dyn ObjectRepository,
    random: &mut dyn Randomness,
    categories: &Categories,
    input: NewTask,
) -> Result<Oid, UseCaseError> {
    categories.check(&input.labels).map_err(Refusal::Labels)?;
    let oid = Oid::generate(&mut |b| random.fill(b));
    let mut task = Task::new(oid.clone(), input.subject);
    task.base.path = input.path;
    task.base.labels = input.labels;
    task.base.body = input.body;
    task.priority = input.priority;
    task.due = input.due;
    task.deadline = input.deadline;
    objects.put(&Object::Task(task))?;
    Ok(oid)
}

pub fn new_event(
    objects: &dyn ObjectRepository,
    random: &mut dyn Randomness,
    categories: &Categories,
    input: NewEvent,
) -> Result<Oid, UseCaseError> {
    categories.check(&input.labels).map_err(Refusal::Labels)?;
    let oid = Oid::generate(&mut |b| random.fill(b));
    let mut event = Event::new(oid.clone(), input.subject, input.start, input.end);
    event.base.path = input.path;
    event.base.labels = input.labels;
    event.base.body = input.body;
    objects.put(&Object::Event(event))?;
    Ok(oid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{FixedRandom, MemoryStore, oid};
    use dam_domain::Category;

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

    #[test]
    fn a_new_task_lands_in_working_with_a_generated_oid_and_nothing_staged() {
        let store = MemoryStore::new();
        let id = new_task(
            &store,
            &mut FixedRandom(7),
            &Categories::default(),
            input("milk", &[]),
        )
        .unwrap();
        assert_eq!(id, oid(7));
        let got = store.get(&id).unwrap().unwrap();
        assert_eq!(got.base().subject, "milk");
        assert!(store.staged().unwrap().is_empty());
        assert!(store.committed(&id).unwrap().is_none());
    }

    #[test]
    fn labels_are_checked_against_categories() {
        let store = MemoryStore::new();
        let err = new_task(
            &store,
            &mut FixedRandom(1),
            &effort(),
            input("x", &["light", "deep"]),
        )
        .unwrap_err();
        assert!(matches!(err, UseCaseError::Refused(Refusal::Labels(_))));
        assert!(store.all().unwrap().is_empty());
    }

    #[test]
    fn a_new_event_lands_in_working() {
        let store = MemoryStore::new();
        let start = When::Day(jiff::civil::date(2026, 9, 25));
        let end = When::Day(jiff::civil::date(2026, 9, 26));
        let id = new_event(
            &store,
            &mut FixedRandom(2),
            &Categories::default(),
            NewEvent {
                subject: "dentist".into(),
                path: Path::root(),
                start,
                end,
                labels: Default::default(),
                body: String::new(),
            },
        )
        .unwrap();
        assert!(matches!(store.get(&id).unwrap(), Some(Object::Event(_))));
    }
}
