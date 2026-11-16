use std::collections::BTreeSet;

use dam_domain::{CallerId, Date, Event, Object, Oid, Path, Permission, Priority, Task, When};

use crate::errors::{Refusal, Retry, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::land::Landed;
use crate::use_cases::placement::Placement;

mod by_id;
mod project_task;

pub struct NewTask {
    pub parent: Option<Oid>,
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
    pub parent: Option<Oid>,
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
            self.entering(&[], || {
                let oid = Oid::generate(&mut |b| self.random.fill(b));
                let mut task = Task::new(oid, input.subject);
                task.base.labels = input.labels;
                task.base.body = input.body;
                task.priority = input.priority;
                task.due = input.due;
                task.deadline = input.deadline;
                task.base.path = input.path;
                task.base.parent = input.parent;
                self.land_new(input.id.as_ref(), Object::Task(task))
            })
        })
    }

    pub fn new_event(&self, input: NewEvent) -> Result<Landed, UseCaseError> {
        atomically(self.store, || {
            let oid = Oid::generate(&mut |b| self.random.fill(b));
            let mut event = Event::new(oid, input.subject, input.start, input.end);
            event.base.labels = input.labels;
            event.base.body = input.body;
            event.base.path = input.path;
            event.base.parent = input.parent;
            self.land_new(input.id.as_ref(), Object::Event(event))
        })
    }

    pub(crate) fn placed_new(&self, object: Object) -> Result<Object, UseCaseError> {
        self.placed(object, true)
    }

    pub(crate) fn placed_again(&self, object: Object) -> Result<Object, UseCaseError> {
        if object.base().parent.is_none() {
            return Ok(object);
        }
        self.placed(object, false)
    }

    fn placed(&self, mut object: Object, creating: bool) -> Result<Object, UseCaseError> {
        let (path, parent) = (object.base().path.clone(), object.base().parent.clone());
        let asked = Placement {
            path: Some(&path),
            parent: parent.as_ref(),
            writing: creating.then_some(&object),
        };
        let (path, parent) = self.placement(asked, || Retry::New(object.base().subject.clone()))?;
        object.base_mut().path = path;
        object.base_mut().parent = parent;
        Ok(object)
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
mod project_task_tests;
#[cfg(test)]
mod tests;
