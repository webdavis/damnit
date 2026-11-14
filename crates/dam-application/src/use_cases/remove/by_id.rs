use dam_domain::{Author, CallerId, ItemHash};

use super::Removal;
use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub fn remove_by_id(&self, id: &CallerId) -> Result<Removal, UseCaseError> {
        let Some(oid) = self.store.oid_for_id(&self.id_key(id))? else {
            return Ok(Removal::Nothing);
        };
        match self.access.author() {
            Author::Token(author) => self.withdraw_or_send(author, &ItemHash::new(oid)),
            Author::Owner if self.store.get(&oid)?.is_some() => self.remove_or_send(&oid),
            Author::Owner => Ok(Removal::Nothing),
        }
    }
}
