use dam_domain::{Object, Oid, unsettled};

use crate::errors::{Refusal, UseCaseError};

pub(super) fn settled_around(
    all: Vec<Object>,
    owner: &Object,
) -> Result<Vec<Object>, UseCaseError> {
    let Some(folder) = owner.base().owned_folder() else {
        return Ok(all);
    };
    let mut settled: Vec<(Oid, Oid)> = Vec::new();
    let mut ambiguous: Vec<(Oid, Vec<Oid>)> = Vec::new();
    for (object, could_hold) in unsettled(&all) {
        match could_hold.as_slice() {
            [only] => settled.push((object.oid().clone(), only.clone())),
            _ if object.base().path.is_within(&folder) => {
                ambiguous.push((object.oid().clone(), could_hold));
            }
            _ => {}
        }
    }
    if let Some((_, could_hold)) = ambiguous.first() {
        return Err(Refusal::ParentNotKnown {
            oid: owner.oid().clone(),
            namesakes: could_hold
                .iter()
                .filter(|n| *n != owner.oid())
                .cloned()
                .collect(),
            unsettled: ambiguous.into_iter().map(|(oid, _)| oid).collect(),
            folder,
            held_by_a_remote: false,
        }
        .into());
    }
    Ok(all
        .into_iter()
        .map(|mut object| {
            if let Some((_, parent)) = settled.iter().find(|(oid, _)| oid == object.oid()) {
                object.base_mut().parent = Some(parent.clone());
            }
            object
        })
        .collect())
}
