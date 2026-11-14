use dam_domain::{
    Base, Event, Field, Item, Object, Oid, Outcome, Permission, Task, changed_fields, merge_fields,
};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::items::land::{Answer, Landed, Sent};
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::subtree::refuse_losing_a_held_folder;

impl Dam<'_> {
    pub(super) fn restate(&self, current: Object, fresh: Object) -> Result<Landed, UseCaseError> {
        let change = Permission::change(current.kind());
        let Some(sender) = self.access.token() else {
            let after = restated(&current, fresh)?;
            self.refuse_unpermitted_or_unlabelled(change, Some(&current), &after)?;
            refuse_moving_inside_itself(&current, &after)?;
            refuse_losing_a_held_folder(self.store, &current, &after)?;
            return self.put_or_send(change, Some(&current), after);
        };
        let statement = self.last_statement(sender, current.oid())?;
        let stated = statement
            .as_ref()
            .and_then(|item| item.change.after())
            .unwrap_or(&current);
        let restatement = restated(stated, fresh)?;
        let fields = changed_fields(stated, &restatement);
        if fields.contains(&Field::Path) {
            return Err(Refusal::NotOpenToTokens("mv").into());
        }
        if fields.is_empty() {
            let item = statement.ok_or(ItemRefusal::NothingChanged)?;
            return Ok(Landed::Sent(Sent {
                answer: Answer::Duplicate,
                item,
            }));
        }
        let after = merge_fields(&current, &restatement, &fields);
        self.refuse_unpermitted_or_unlabelled(change, Some(&current), &after)?;
        self.put_or_send(change, Some(&current), after)
    }

    fn last_statement(&self, sender: &str, oid: &Oid) -> Result<Option<Item>, UseCaseError> {
        Ok(self.store.items()?.into_iter().rev().find(|item| {
            item.target() == oid
                && item.author.name == sender
                && item.change.after().is_some()
                && matches!(item.outcome, Outcome::Waiting | Outcome::Added(_))
        }))
    }
}

fn refuse_moving_inside_itself(before: &Object, after: &Object) -> Result<(), UseCaseError> {
    if before
        .base()
        .owned_folder()
        .is_some_and(|folder| after.base().path.is_within(&folder))
    {
        return Err(Refusal::MoveInsideItself(before.oid().clone()).into());
    }
    Ok(())
}

fn restated(before: &Object, fresh: Object) -> Result<Object, UseCaseError> {
    match (before, fresh) {
        (Object::Task(b), Object::Task(f)) => Ok(Object::Task(Task {
            base: restated_base(&b.base, f.base),
            priority: f.priority,
            due: f.due,
            deadline: f.deadline,
            ..b.clone()
        })),
        (Object::Event(b), Object::Event(f)) => Ok(Object::Event(Event {
            base: restated_base(&b.base, f.base),
            start: f.start,
            end: f.end,
            ..b.clone()
        })),
        (Object::Task(_), Object::Event(_)) => {
            Err(Refusal::NotAnEvent(before.oid().clone()).into())
        }
        (Object::Event(_), Object::Task(_)) => Err(Refusal::NotATask(before.oid().clone()).into()),
    }
}

fn restated_base(before: &Base, fresh: Base) -> Base {
    Base {
        subject: fresh.subject,
        body: fresh.body,
        path: fresh.path,
        labels: fresh.labels,
        ..before.clone()
    }
}
