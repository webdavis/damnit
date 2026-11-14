use dam_domain::{Completion, Date, Object, Oid, Priority, Recurrence, When};

use crate::errors::{Refusal, UseCaseError};

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
