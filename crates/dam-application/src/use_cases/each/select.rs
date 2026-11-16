use dam_domain::Oid;

use super::Selector;
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::load::no_such_object;

impl Dam<'_> {
    pub fn select(&self, selector: &Selector) -> Result<Vec<Oid>, UseCaseError> {
        let listed = match selector {
            Selector::Oids(oids) => return self.seen_oids(oids),
            Selector::Filter(name, place, local) => self.listed_by_name(name, place, local)?,
        };
        Ok(listed.into_iter().map(|o| o.oid().clone()).collect())
    }

    fn seen_oids(&self, oids: &[Oid]) -> Result<Vec<Oid>, UseCaseError> {
        let mut seen: Vec<Oid> = Vec::new();
        for oid in oids {
            if self.seen_object(oid)?.is_none() {
                return Err(no_such_object(oid).into());
            }
            if !seen.contains(oid) {
                seen.push(oid.clone());
            }
        }
        Ok(seen)
    }
}
