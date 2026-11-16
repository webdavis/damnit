use super::condition::Condition;
use super::field::Has;
use super::fields::{COMPLETED_AT, DONE, KIND};
use super::filter::Filter;
use crate::Object;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Ls,
    Agenda,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    pub tasks: bool,
    pub events: bool,
    pub done: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultsError {
    KindContradicts {
        field: &'static str,
    },
    DoneContradicts {
        field: &'static str,
    },
    BothKinds {
        task_field: &'static str,
        event_field: &'static str,
    },
    TaskFieldInAgenda {
        field: &'static str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    Drop(&'static str),
    UseAny,
    UseLs,
}

impl DefaultsError {
    pub fn fix(&self) -> Fix {
        match self {
            DefaultsError::KindContradicts { .. } => Fix::Drop(KIND.key),
            DefaultsError::DoneContradicts { .. } => Fix::Drop(DONE.key),
            DefaultsError::BothKinds { .. } => Fix::UseAny,
            DefaultsError::TaskFieldInAgenda { .. } => Fix::UseLs,
        }
    }
}

impl Scope {
    pub fn admits(&self, object: &Object) -> bool {
        match object {
            Object::Task(t) => self.tasks && self.done.is_none_or(|d| t.completion.is_done() == d),
            Object::Event(_) => self.events,
        }
    }
}

pub fn scope(filter: &Filter, command: Command) -> Result<Scope, DefaultsError> {
    let mut top: Vec<&Condition> = filter.conditions().iter().collect();
    if let [alone] = filter.any() {
        top.extend(alone.conditions());
    }
    let named = |has: Has| {
        top.iter()
            .find(|c| c.field.has == has && c.key() != DONE.key)
            .map(|c| c.key())
    };
    let (task_field, event_field) = (named(Has::Task), named(Has::Event));
    let typed = |key: &str, a: &str, b: &str| {
        let typed: Vec<&&Condition> = top.iter().filter(|c| c.key() == key).collect();
        (!typed.is_empty()).then(|| {
            let admits = |w| typed.iter().all(|c| c.admits_word(w));
            (admits(a), admits(b))
        })
    };
    let kind = typed(KIND.key, "task", "event");
    let done = typed(DONE.key, "true", "false");
    let completed = top
        .iter()
        .find(|c| c.key() == COMPLETED_AT.key && c.text() != "none")
        .map(|c| c.key());
    if let (Some(task_field), Some(event_field)) = (task_field, event_field) {
        return Err(DefaultsError::BothKinds {
            task_field,
            event_field,
        });
    }
    if let (Some(field), Some((_, false))) = (event_field, kind) {
        return Err(DefaultsError::KindContradicts { field });
    }
    if let (Some(field), Some((false, _))) = (task_field, kind) {
        return Err(DefaultsError::KindContradicts { field });
    }
    if let (Some(field), Some((false, true))) = (completed, done) {
        return Err(DefaultsError::DoneContradicts { field });
    }
    if let (Some(field), Some((yes, no))) = (event_field, done)
        && yes != no
    {
        return Err(DefaultsError::DoneContradicts { field });
    }
    if command == Command::Agenda {
        if let Some(field) = task_field {
            return Err(DefaultsError::TaskFieldInAgenda { field });
        }
        if kind == Some((true, false)) {
            return Err(DefaultsError::TaskFieldInAgenda { field: KIND.key });
        }
    }
    let (tasks, events) = kind
        .or(event_field.map(|_| (false, true)))
        .or(task_field.map(|_| (true, false)))
        .or(names_both_kinds(filter).then_some((true, true)))
        .unwrap_or((command == Command::Ls, command == Command::Agenda));
    let done = match (done, completed) {
        (Some((yes, no)), _) => (yes != no).then_some(yes),
        (None, Some(_)) => Some(true),
        (None, None) => (command == Command::Ls).then_some(false),
    };
    Ok(Scope {
        tasks,
        events,
        done,
    })
}

fn names_both_kinds(filter: &Filter) -> bool {
    let mut has = Vec::new();
    collect(filter.any(), &mut has);
    has.contains(&Has::Task) && has.contains(&Has::Event)
}

fn collect(any: &[Filter], has: &mut Vec<Has>) {
    for f in any {
        has.extend(f.conditions().iter().map(|c| c.field.has));
        collect(f.any(), has);
    }
}
