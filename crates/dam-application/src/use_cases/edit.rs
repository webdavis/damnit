use dam_domain::{Categories, Date, Object, Oid, Priority, Rule, When, cycle_in};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;

/// Outer `None` leaves a field alone; `Some(None)` clears it; `Some(Some(v))` sets it.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct EditFields {
    pub subject: Option<String>,
    pub body: Option<String>,
    pub priority: Option<Priority>,
    pub due: Option<Option<When>>,
    pub deadline: Option<Option<Date>>,
    pub add_labels: Vec<String>,
    pub remove_labels: Vec<String>,
    pub add_depends: Vec<Oid>,
    pub remove_depends: Vec<Oid>,
    pub recurrence: Option<Option<String>>,
    pub attach: Option<Option<Oid>>,
    pub start: Option<When>,
    pub end: Option<When>,
    pub location: Option<Option<String>>,
}

pub fn edit(
    objects: &dyn ObjectRepository,
    categories: &Categories,
    oid: &Oid,
    fields: EditFields,
) -> Result<Object, UseCaseError> {
    let current = objects
        .get(oid)?
        .ok_or_else(|| Refusal::NoSuchObject(oid.short().to_string()))?;
    let next = apply(&current, &fields)?;
    categories
        .check(&next.base().labels)
        .map_err(Refusal::Labels)?;
    if !fields.add_depends.is_empty() {
        let edges = |o: &Oid| {
            objects
                .get(o)
                .ok()
                .flatten()
                .map(|x| x.base().depends.clone())
                .unwrap_or_default()
        };
        if let Some(path) = cycle_in(&fields.add_depends, oid, &edges) {
            return Err(Refusal::Cycle {
                oid: oid.clone(),
                path,
            }
            .into());
        }
    }
    if let Some(Some(event)) = &fields.attach {
        match objects.get(event)? {
            Some(Object::Event(_)) => {}
            _ => return Err(Refusal::NoSuchObject(event.short().to_string()).into()),
        }
    }
    objects.put(&next)?;
    Ok(next)
}

/// Apply `fields` to a copy without writing; the editor round trip and the cli preview use it.
pub fn apply(object: &Object, fields: &EditFields) -> Result<Object, UseCaseError> {
    let mut next = object.clone();
    let is_task = matches!(next, Object::Task(_));
    if !is_task
        && (fields.priority.is_some()
            || fields.due.is_some()
            || fields.deadline.is_some()
            || fields.attach.is_some()
            || fields.recurrence.is_some())
    {
        return Err(Refusal::NotATask(next.oid().clone()).into());
    }
    if is_task && (fields.start.is_some() || fields.end.is_some() || fields.location.is_some()) {
        return Err(UseCaseError::Parse("start applies to events".into()));
    }

    let base = next.base_mut();
    if let Some(s) = &fields.subject {
        base.subject = s.clone();
    }
    if let Some(b) = &fields.body {
        base.body = b.clone();
    }
    for l in &fields.remove_labels {
        base.labels.remove(l);
    }
    for l in &fields.add_labels {
        base.labels.insert(l.clone());
    }
    base.depends.retain(|d| !fields.remove_depends.contains(d));
    for d in &fields.add_depends {
        if !base.depends.contains(d) {
            base.depends.push(d.clone());
        }
    }
    if let Some(r) = &fields.recurrence {
        if let Some(text) = r {
            Rule::parse(text).map_err(|e| UseCaseError::Parse(e.to_string()))?;
        }
        base.recurrence = r.clone();
    }

    match &mut next {
        Object::Task(t) => {
            if let Some(p) = fields.priority {
                t.priority = p;
            }
            if let Some(d) = &fields.due {
                t.due = d.clone();
            }
            if let Some(d) = &fields.deadline {
                t.deadline = *d;
            }
            if let Some(a) = &fields.attach {
                t.event = a.clone();
            }
        }
        Object::Event(e) => {
            if let Some(s) = &fields.start {
                e.start = s.clone();
            }
            if let Some(x) = &fields.end {
                e.end = x.clone();
            }
            if let Some(l) = &fields.location {
                e.location = l.clone();
            }
        }
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{MemoryStore, oid};
    use dam_domain::{Categories, Category, Event, Object, Task, When};
    use jiff::civil::date;

    fn seed(store: &MemoryStore, byte: u8) -> Oid {
        store
            .put(&Object::Task(Task::new(oid(byte), format!("t{byte}"))))
            .unwrap();
        oid(byte)
    }

    #[test]
    fn subject_priority_and_due_are_set() {
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        let out = edit(
            &store,
            &Categories::default(),
            &id,
            EditFields {
                subject: Some("new".into()),
                priority: Some(Priority::HIGHEST),
                due: Some(Some(When::Day(date(2026, 9, 25)))),
                ..EditFields::default()
            },
        )
        .unwrap();
        let t = out.as_task().unwrap();
        assert_eq!(t.base.subject, "new");
        assert_eq!(t.priority, Priority::HIGHEST);
        assert_eq!(t.due, Some(When::Day(date(2026, 9, 25))));
        assert_eq!(store.get(&id).unwrap(), Some(out));
    }

    #[test]
    fn some_none_clears_a_date() {
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        edit(
            &store,
            &Categories::default(),
            &id,
            EditFields {
                due: Some(Some(When::Day(date(2026, 9, 25)))),
                ..Default::default()
            },
        )
        .unwrap();
        let out = edit(
            &store,
            &Categories::default(),
            &id,
            EditFields {
                due: Some(None),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(out.as_task().unwrap().due, None);
    }

    #[test]
    fn labels_are_added_removed_and_checked() {
        let cats = Categories::new(vec![Category {
            name: "effort".into(),
            values: vec!["light".into(), "deep".into()],
            exclusive: true,
        }])
        .unwrap();
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        edit(
            &store,
            &cats,
            &id,
            EditFields {
                add_labels: vec!["deep".into(), "errand".into()],
                ..Default::default()
            },
        )
        .unwrap();
        let err = edit(
            &store,
            &cats,
            &id,
            EditFields {
                add_labels: vec!["light".into()],
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(matches!(err, UseCaseError::Refused(Refusal::Labels(_))));
        let out = edit(
            &store,
            &cats,
            &id,
            EditFields {
                remove_labels: vec!["deep".into()],
                add_labels: vec!["light".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(out.base().labels.contains("light") && !out.base().labels.contains("deep"));
    }

    #[test]
    fn a_dependency_cycle_is_refused() {
        let store = MemoryStore::new();
        let a = seed(&store, 1);
        let b = seed(&store, 2);
        edit(
            &store,
            &Categories::default(),
            &b,
            EditFields {
                add_depends: vec![a.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        let err = edit(
            &store,
            &Categories::default(),
            &a,
            EditFields {
                add_depends: vec![b.clone()],
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(
            err,
            UseCaseError::Refused(Refusal::Cycle {
                oid: a,
                path: vec![b, oid(1)]
            })
        );
    }

    #[test]
    fn a_bad_recurrence_is_a_parse_error_and_a_good_one_is_kept() {
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        let err = edit(
            &store,
            &Categories::default(),
            &id,
            EditFields {
                recurrence: Some(Some("weekly".into())),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(matches!(err, UseCaseError::Parse(_)));
        let out = edit(
            &store,
            &Categories::default(),
            &id,
            EditFields {
                recurrence: Some(Some("every! week".into())),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(out.base().recurrence.as_deref(), Some("every! week"));
    }

    #[test]
    fn attach_needs_an_existing_event() {
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        let err = edit(
            &store,
            &Categories::default(),
            &id,
            EditFields {
                attach: Some(Some(oid(9))),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            UseCaseError::Refused(Refusal::NoSuchObject(_))
        ));
        store
            .put(&Object::Event(Event::new(
                oid(9),
                "e",
                When::Day(date(2026, 1, 1)),
                When::Day(date(2026, 1, 2)),
            )))
            .unwrap();
        let out = edit(
            &store,
            &Categories::default(),
            &id,
            EditFields {
                attach: Some(Some(oid(9))),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(out.as_task().unwrap().event, Some(oid(9)));
    }

    #[test]
    fn task_fields_on_an_event_are_refused() {
        let store = MemoryStore::new();
        store
            .put(&Object::Event(Event::new(
                oid(9),
                "e",
                When::Day(date(2026, 1, 1)),
                When::Day(date(2026, 1, 2)),
            )))
            .unwrap();
        let err = edit(
            &store,
            &Categories::default(),
            &oid(9),
            EditFields {
                priority: Some(Priority::HIGHEST),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err, UseCaseError::Refused(Refusal::NotATask(oid(9))));
    }
}
