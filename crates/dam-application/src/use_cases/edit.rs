use std::collections::BTreeMap;

use dam_domain::{Completion, Date, Object, Oid, Permission, Priority, Recurrence, When, cycle_in};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::land::Landed;
use crate::use_cases::load::no_such_object;
use crate::use_cases::subtree::refuse_losing_a_held_folder;

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
    pub estimate: Option<Option<u32>>,
    pub focus: Option<Option<Date>>,
}

impl Dam<'_> {
    pub fn edit(&self, oid: &Oid, fields: EditFields) -> Result<Landed, UseCaseError> {
        atomically(self.store, || {
            let current = self.load_permitted(oid, Permission::change)?;
            self.refuse_what_a_token_may_not_edit(&fields)?;
            let mut next = apply(&current, &fields)?;
            if let Some(day) = fields.focus {
                self.mark_focus(&mut next, day)?;
            }
            self.config
                .categories
                .check(&next.base().labels)
                .map_err(Refusal::Labels)?;
            if self.access.token().is_none()
                && let Some(path) = self.cycle_through(oid, &fields.add_depends)?
            {
                return Err(Refusal::Cycle {
                    oid: oid.clone(),
                    path,
                }
                .into());
            }
            if let Some(Some(event)) = &fields.attach
                && !matches!(self.store.get(event)?, Some(Object::Event(_)))
            {
                return Err(no_such_object(event).into());
            }
            refuse_losing_a_held_folder(self.store, &current, &next)?;
            self.put_or_send(Permission::change(current.kind()), Some(&current), next)
        })
    }

    pub(crate) fn cycle_through(
        &self,
        oid: &Oid,
        added: &[Oid],
    ) -> Result<Option<Vec<Oid>>, UseCaseError> {
        let edges = depends_reachable_from(self.store, added)?;
        let edges_of = |o: &Oid| edges.get(o).cloned().unwrap_or_default();
        Ok(cycle_in(added, oid, &edges_of))
    }
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
            || fields.estimate.is_some()
            || fields.focus.is_some()
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
            if let Some(e) = fields.estimate {
                t.estimate = e;
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

mod focus;
mod token_limits;

#[cfg(test)]
mod tests;
