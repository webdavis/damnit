use dam_domain::{Author, Change, Field, Item, Object, Oid, Outcome, changed_fields, merge_fields};

use super::refusal::ItemRefusal;
use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub(crate) fn unpick(&self, oid: &Oid) -> Result<(), UseCaseError> {
        let Some(item) = self.added_item_staged_at(oid)? else {
            return Ok(());
        };
        let working = self.store.get(oid)?;
        match (&item.change, working) {
            (Change::Create { after }, Some(working)) if working == *after => {
                self.store.delete(oid)?;
            }
            (Change::Create { .. }, Some(_)) => {
                return Err(ItemRefusal::EditedSinceAdded {
                    hash: item.hash.clone(),
                    target: oid.clone(),
                }
                .into());
            }
            (Change::Update { before, after }, Some(working)) => {
                if let Some(committed) = self.store.committed(oid)? {
                    let fields = still_as_the_item_left_them(before, after, &working);
                    self.store
                        .put(&merge_fields(&working, &committed, &fields))?;
                }
            }
            (Change::Delete { .. }, None) => {
                if let Some(committed) = self.store.committed(oid)? {
                    self.store.put(&committed)?;
                }
            }
            _ => {}
        }
        self.store.settle(&item.hash, Outcome::Waiting)?;
        Ok(())
    }

    pub(super) fn added_item_staged_at(&self, oid: &Oid) -> Result<Option<Item>, UseCaseError> {
        let Some(author) = self
            .store
            .staged()?
            .into_iter()
            .find_map(|s| match s.author {
                Author::Token(author) if s.change.oid() == oid => Some(author),
                _ => None,
            })
        else {
            return Ok(None);
        };
        Ok(self.store.items()?.into_iter().rev().find(|item| {
            matches!(item.outcome, Outcome::Added(_))
                && item.target() == oid
                && item.author == author
        }))
    }
}

fn still_as_the_item_left_them(before: &Object, after: &Object, working: &Object) -> Vec<Field> {
    let moved_since = changed_fields(after, working);
    changed_fields(before, after)
        .into_iter()
        .filter(|f| !moved_since.contains(f))
        .collect()
}
