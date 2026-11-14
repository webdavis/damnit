use dam_domain::{Author, ItemHash, Permission, TokenAuthor};

use super::{ItemAndTask, Removal};
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::refusal::ItemRefusal;

impl Dam<'_> {
    pub fn remove_item(&self, hash: &ItemHash) -> Result<Removal, UseCaseError> {
        match self.access.author() {
            Author::Token(author) => self.withdraw_or_send(author, hash),
            Author::Owner => Ok(Removal::ItemDeclined(Box::new(self.dismiss(hash)?))),
        }
    }

    pub(super) fn withdraw_or_send(
        &self,
        author: TokenAuthor,
        hash: &ItemHash,
    ) -> Result<Removal, UseCaseError> {
        atomically(self.store, || {
            let item = self.store.item(hash)?;
            if let Some(item) = item.as_ref().filter(|item| item.is_waiting()) {
                if item.author.name != author.name {
                    return Err(ItemRefusal::NotYours(hash.clone()).into());
                }
                self.dismiss(hash)?;
                return Ok(Removal::Withdrawn(ItemAndTask {
                    item: hash.clone(),
                    oid: item.target().clone(),
                }));
            }
            let target = item.map_or_else(|| hash.oid().clone(), |item| item.target().clone());
            if !self
                .store
                .get(&target)?
                .is_some_and(|o| self.access.sees(&o))
            {
                return Ok(Removal::Nothing);
            }
            let object = self.load_permitted(&target, Permission::remove)?;
            self.send_removal(author, object)
        })
    }
}
