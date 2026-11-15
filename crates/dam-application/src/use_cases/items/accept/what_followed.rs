use dam_domain::{Author, Authored, Item, Object, Oid};

use crate::errors::UseCaseError;
use crate::ports::{FollowOn, ObjectState};
use crate::service::Dam;

pub(super) struct Untouched {
    objects: Vec<Object>,
    staged: Vec<Authored>,
}

impl Dam<'_> {
    pub(super) fn untouched(&self) -> Result<Untouched, UseCaseError> {
        Ok(Untouched {
            objects: self.store.all()?,
            staged: self.store.staged()?,
        })
    }

    pub(super) fn keep_what_followed(
        &self,
        item: &Item,
        untouched: Untouched,
    ) -> Result<(), UseCaseError> {
        let staged = self.store.staged()?;
        let mut follow_ons = Vec::new();
        for was in untouched.objects {
            if was.oid() == item.target() {
                continue;
            }
            let Some(now) = self.store.get(was.oid())?.filter(|now| *now != was) else {
                continue;
            };
            follow_ons.push(FollowOn {
                before: state(was, &untouched.staged),
                after: state(now, &staged),
            });
        }
        if !follow_ons.is_empty() {
            self.store.keep_follow_ons(&item.hash, &follow_ons)?;
        }
        Ok(())
    }
}

pub(crate) fn state(working: Object, staged: &[Authored]) -> ObjectState {
    ObjectState {
        owners_staged: owners_entry(staged, working.oid()),
        working,
    }
}

fn owners_entry(staged: &[Authored], oid: &Oid) -> Option<Authored> {
    staged
        .iter()
        .find(|s| s.author == Author::Owner && s.change.oid() == oid)
        .cloned()
}
