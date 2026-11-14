use std::collections::BTreeSet;

use dam_domain::{CallerId, Date, Event, Object, Oid, Path, Permission, Priority, Task, When};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::land::Landed;

mod by_id;

pub struct NewTask {
    pub subject: String,
    pub path: Path,
    pub priority: Priority,
    pub due: Option<When>,
    pub deadline: Option<Date>,
    pub labels: BTreeSet<String>,
    pub body: String,
    pub id: Option<CallerId>,
}

pub struct NewEvent {
    pub subject: String,
    pub path: Path,
    pub start: When,
    pub end: When,
    pub labels: BTreeSet<String>,
    pub body: String,
    pub id: Option<CallerId>,
}

impl Dam<'_> {
    pub fn new_task(&self, input: NewTask) -> Result<Landed, UseCaseError> {
        atomically(self.store, || {
            let oid = Oid::generate(&mut |b| self.random.fill(b));
            let mut task = Task::new(oid, input.subject);
            task.base.path = input.path;
            task.base.labels = input.labels;
            task.base.body = input.body;
            task.priority = input.priority;
            task.due = input.due;
            task.deadline = input.deadline;
            self.land_new(input.id.as_ref(), Object::Task(task))
        })
    }

    pub fn new_event(&self, input: NewEvent) -> Result<Landed, UseCaseError> {
        atomically(self.store, || {
            let oid = Oid::generate(&mut |b| self.random.fill(b));
            let mut event = Event::new(oid, input.subject, input.start, input.end);
            event.base.path = input.path;
            event.base.labels = input.labels;
            event.base.body = input.body;
            self.land_new(input.id.as_ref(), Object::Event(event))
        })
    }

    fn refuse_unpermitted_or_unlabelled(
        &self,
        permission: Permission,
        before: Option<&Object>,
        object: &Object,
    ) -> Result<(), UseCaseError> {
        self.refuse_unless_permitted(permission, before, object)?;
        self.config
            .categories
            .check(&object.base().labels)
            .map_err(Refusal::Labels)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
