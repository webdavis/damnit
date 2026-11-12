use dam_domain::{
    Author, Base, CallerId, Event, Field, IdHolder, IdKey, Item, ItemHash, Object, Oid, Outcome,
    Permission, Task, changed_fields, merge_fields,
};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::items::land::{Answer, Landed, Sent};
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::subtree::refuse_losing_a_held_folder;

impl Dam<'_> {
    pub(crate) fn land_new(
        &self,
        id: Option<&CallerId>,
        fresh: Object,
    ) -> Result<Landed, UseCaseError> {
        let create = Permission::create(fresh.kind());
        let Some(id) = id else {
            self.refuse_unpermitted_or_unlabelled(create, None, &fresh)?;
            return self.put_or_send(create, None, fresh);
        };
        let key = self.id_key(id);
        let mapped = self.store.oid_for_id(&key)?;
        if let Some(oid) = &mapped
            && let Some(before) = self.own_earlier(oid)?
        {
            return self.restate(before, fresh);
        }
        self.refuse_unpermitted_or_unlabelled(create, None, &fresh)?;
        let landed = self.put_or_send_among(create, None, fresh, &|earlier| {
            Some(earlier.target()) == mapped.as_ref()
        })?;
        self.store.map_id(&key, landed.oid())?;
        Ok(landed)
    }

    pub fn caller_id(&self, oid: &Oid) -> Result<Option<CallerId>, UseCaseError> {
        Ok(self.store.id_for_oid(&self.id_holder(), oid)?)
    }

    fn id_holder(&self) -> IdHolder {
        match self.access.author() {
            Author::Owner => IdHolder::Owner,
            Author::Token(token) => IdHolder::Token(token.name),
        }
    }

    fn id_key(&self, id: &CallerId) -> IdKey {
        IdKey {
            holder: self.id_holder(),
            session: self
                .access
                .details()
                .and_then(|d| d.session())
                .map(str::to_string),
            id: id.clone(),
        }
    }

    fn own_earlier(&self, oid: &Oid) -> Result<Option<Object>, UseCaseError> {
        if let Some(object) = self.store.get(oid)?.filter(|o| self.access.sees(o)) {
            return Ok(Some(object));
        }
        Ok(self
            .store
            .item(&ItemHash::new(oid.clone()))?
            .filter(|item| {
                item.is_waiting() && self.access.token() == Some(item.author.name.as_str())
            })
            .and_then(|item| item.change.after().cloned()))
    }

    fn restate(&self, current: Object, fresh: Object) -> Result<Landed, UseCaseError> {
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

#[cfg(test)]
mod tests;
