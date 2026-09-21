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
    /// Reopens a completed task; refused on one that is already open.
    pub undone: bool,
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
            || fields.recurrence.is_some()
            || fields.undone)
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
            if fields.undone {
                if !t.done {
                    return Err(Refusal::NotCompleted(t.base.oid.clone()).into());
                }
                t.done = false;
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
mod tests;
