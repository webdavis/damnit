use std::collections::BTreeSet;

use dam_domain::Object;

use super::settled_around;
use crate::errors::UseCaseError;
use crate::ports::ObjectRepository;

pub(super) fn held_chain(
    objects: &dyn ObjectRepository,
    owner: &Object,
) -> Result<Vec<Object>, UseCaseError> {
    let all = settled_around(objects.all()?, owner)?;
    let mut holders = BTreeSet::from([owner.oid().clone()]);
    let mut chain = Vec::new();
    loop {
        let found: Vec<Object> = all
            .iter()
            .filter(|o| !holders.contains(o.oid()))
            .filter(|o| {
                o.base()
                    .parent
                    .as_ref()
                    .is_some_and(|p| holders.contains(p))
            })
            .cloned()
            .collect();
        if found.is_empty() {
            return Ok(chain);
        }
        holders.extend(found.iter().map(|o| o.oid().clone()));
        chain.extend(found);
    }
}
