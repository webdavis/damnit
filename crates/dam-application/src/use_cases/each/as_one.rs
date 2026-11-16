use dam_domain::{ItemHash, Oid};

use super::SetChange;
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::items::land::Answer;

impl Dam<'_> {
    pub(super) fn gathered(
        &self,
        mut changes: Vec<SetChange>,
    ) -> Result<Vec<SetChange>, UseCaseError> {
        let set = self.unused_item_hash()?;
        let fresh = changes
            .iter_mut()
            .filter_map(|change| change.sent.as_mut())
            .filter(|sent| sent.answer != Answer::Duplicate);
        for sent in fresh {
            self.store.put_in_set(&sent.item.hash, &set)?;
            sent.item.set = Some(set.clone());
        }
        Ok(changes)
    }

    fn unused_item_hash(&self) -> Result<ItemHash, UseCaseError> {
        loop {
            let hash = ItemHash::new(Oid::generate(&mut |b| self.random.fill(b)));
            if self.store.item(&hash)?.is_none() {
                return Ok(hash);
            }
        }
    }
}
