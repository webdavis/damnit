use std::collections::BTreeMap;

use dam_domain::{Categories, Completion, Date, Object, Oid, Priority, Recurrence, When, cycle_in};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::{load, no_such_object};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct EditFields {
    pub subject: Option<String>,
    pub body: Option<String>,
    pub priority: Option<Priority>,
    pub due: Option<Option<When>>,
    pub deadline: Option<Option<Date>>,
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
    atomically(objects, || {
        let current = load(objects, oid)?;
        let next = apply(&current, &fields)?;
        categories
            .check(&next.base().labels)
            .map_err(Refusal::Labels)?;
        if !fields.add_depends.is_empty() {
            let edges = depends_reachable_from(objects, &fields.add_depends)?;
            let edges_of = |o: &Oid| edges.get(o).cloned().unwrap_or_default();
            if let Some(path) = cycle_in(&fields.add_depends, oid, &edges_of) {
                return Err(Refusal::Cycle {
                    oid: oid.clone(),
                    path,
                }
                .into());
            }
        }
        if let Some(Some(event)) = &fields.attach
            && !matches!(objects.get(event)?, Some(Object::Event(_)))
        {
            return Err(no_such_object(event).into());
        }
        objects.put(&next)?;
        Ok(next)
    })
}

fn depends_reachable_from(
    objects: &dyn ObjectRepository,
    starts: &[Oid],
) -> Result<BTreeMap<Oid, Vec<Oid>>, UseCaseError> {
    let mut edges = BTreeMap::new();
    let mut unvisited = starts.to_vec();
    while let Some(at) = unvisited.pop() {
        if edges.contains_key(&at) {
            continue;
        }
        let depends = objects
            .get(&at)?
            .map(|o| o.base().depends.clone())
            .unwrap_or_default();
        unvisited.extend(depends.iter().cloned());
        edges.insert(at, depends);
    }
    Ok(edges)
}

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
        return Err(Refusal::NotAnEvent(next.oid().clone()).into());
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
        base.recurrence = r
            .as_deref()
            .map(Recurrence::dam)
            .transpose()
            .map_err(|e| UseCaseError::Parse(e.to_string()))?;
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
                if !t.completion.is_done() {
                    return Err(Refusal::NotCompleted(t.base.oid.clone()).into());
                }
                t.completion = Completion::Open;
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
