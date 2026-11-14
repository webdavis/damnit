use dam_domain::{Author, Authored, Change, Object, Oid, changed_fields, merge_fields};

use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub(super) fn stage_what_followed(
        &self,
        untouched: Vec<Object>,
    ) -> Result<Vec<Authored>, UseCaseError> {
        let mut staged = Vec::new();
        for was in untouched {
            let Some(now) = self.store.get(was.oid())?.filter(|now| *now != was) else {
                continue;
            };
            let (base, current) = self.owners_staged_base(was.oid())?;
            let Some(current) = current else {
                continue;
            };
            let after = merge_fields(&current, &now, &changed_fields(&was, &now));
            if let Some(change) = Change::between(base, Some(after)) {
                let authored = Authored::by_owner(change).with_details(self.access.details());
                self.store.put_staged(authored.clone())?;
                staged.push(authored);
            }
        }
        Ok(staged)
    }

    fn owners_staged_base(
        &self,
        oid: &Oid,
    ) -> Result<(Option<Object>, Option<Object>), UseCaseError> {
        let staged: Vec<Authored> = self
            .store
            .staged()?
            .into_iter()
            .filter(|s| s.change.oid() == oid)
            .collect();
        if let Some(mine) = staged.iter().find(|s| s.author == Author::Owner) {
            return Ok((mine.change.before().cloned(), mine.change.after().cloned()));
        }
        let base = match staged.iter().find(|s| s.author != Author::Owner) {
            Some(sent) => sent.change.after().cloned(),
            None => self.store.committed(oid)?,
        };
        Ok((base.clone(), base))
    }
}
