use dam_domain::{
    Author, Change, Item, Object, Outcome, Permission, Subject, TokenAuthor, same_proposal,
};

use super::refusal::ItemRefusal;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;

#[allow(
    clippy::large_enum_variant,
    reason = "a landed write is handed straight to the caller, never stored in bulk"
)]
#[derive(Debug, PartialEq, Eq)]
pub enum Landed {
    Written(Object),
    Sent(Sent),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Sent {
    pub answer: Answer,
    pub item: Item,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    New,
    Changed,
    Duplicate,
}

impl Answer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Answer::New => "new",
            Answer::Changed => "changed",
            Answer::Duplicate => "duplicate",
        }
    }
}

impl Dam<'_> {
    pub(crate) fn put_or_send(
        &self,
        permission: Permission,
        before: Option<&Object>,
        after: Object,
    ) -> Result<Landed, UseCaseError> {
        self.refuse_unless_permitted(permission, before, &after)?;
        let Author::Token(author) = self.access.author() else {
            self.store.put(&after)?;
            return Ok(Landed::Written(after));
        };
        let change =
            Change::between(before.cloned(), Some(after)).ok_or(ItemRefusal::NothingChanged)?;
        self.send(author, change)
    }

    pub(crate) fn refuse_unless_permitted(
        &self,
        permission: Permission,
        before: Option<&Object>,
        after: &Object,
    ) -> Result<(), UseCaseError> {
        let touched: Vec<Subject<'_>> = before
            .into_iter()
            .chain([after])
            .map(Subject::Object)
            .collect();
        if self.access.permits(permission, &touched) {
            Ok(())
        } else {
            Err(Refusal::NotPermitted(permission).into())
        }
    }

    fn send(&self, author: TokenAuthor, change: Change) -> Result<Landed, UseCaseError> {
        for earlier in self.store.items()? {
            if earlier.author.name != author.name || !same_proposal(&earlier.change, &change) {
                continue;
            }
            match earlier.outcome {
                Outcome::Waiting => {
                    return Ok(Landed::Sent(Sent {
                        answer: Answer::Duplicate,
                        item: earlier,
                    }));
                }
                Outcome::Declined(_) => return Err(ItemRefusal::Declined(earlier.hash).into()),
                Outcome::Added(_) | Outcome::Withdrawn(_) => {}
            }
        }
        let item = Item::propose(author, change, self.clock.now(), &mut |b| {
            self.random.fill(b)
        });
        self.store.put_item(&item)?;
        let answer = match item.change {
            Change::Create { .. } => Answer::New,
            Change::Update { .. } | Change::Delete { .. } => Answer::Changed,
        };
        Ok(Landed::Sent(Sent { answer, item }))
    }
}

#[cfg(test)]
mod tests;
