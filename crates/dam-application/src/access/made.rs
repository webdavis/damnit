use dam_domain::{Author, Op};

use super::Holder;
use crate::ports::{Store, StoreError};
use crate::use_cases::origins::origins_in;

impl Holder {
    pub fn knowing_what_it_made(mut self, store: &dyn Store) -> Result<Holder, StoreError> {
        let name = self.name.as_str();
        let made_by_name = |author: &Author| matches!(author, Author::Token(t) if t.name == name);
        let mut made: Vec<_> = origins_in(store)?
            .into_iter()
            .filter(|(_, origin)| made_by_name(&origin.author))
            .map(|(oid, _)| oid)
            .collect();
        made.extend(
            store
                .items()?
                .into_iter()
                .filter(|item| {
                    item.is_waiting() && item.author.name == name && item.change.op() == Op::Create
                })
                .map(|item| item.change.oid().clone()),
        );
        self.grants.made = made.into_iter().collect();
        Ok(self)
    }
}
