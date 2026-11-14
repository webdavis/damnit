use dam_domain::{Change, Object, TokenAuthor};

use super::{ItemAndTask, Removal};
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::refusal::ItemRefusal;

impl Dam<'_> {
    pub(super) fn send_removal(
        &self,
        author: TokenAuthor,
        object: Object,
    ) -> Result<Removal, UseCaseError> {
        let oid = object.oid().clone();
        atomically(self.store, || {
            let removal = Change::Delete {
                before: object.clone(),
            };
            match self.send(author.clone(), removal, &|_| true) {
                Ok(sent) => Ok(Removal::Waiting(ItemAndTask {
                    item: sent.item.hash,
                    oid: oid.clone(),
                })),
                Err(UseCaseError::Refused(Refusal::Item(ItemRefusal::Declined(item)))) => {
                    Ok(Removal::Declined(ItemAndTask {
                        item,
                        oid: oid.clone(),
                    }))
                }
                Err(other) => Err(other),
            }
        })
    }
}
