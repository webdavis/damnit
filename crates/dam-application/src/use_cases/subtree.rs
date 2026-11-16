use std::collections::BTreeSet;

use dam_domain::{Object, Oid, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::use_cases::load::load;
use crate::use_cases::parents::placed;

pub(crate) fn contents(
    objects: &dyn ObjectRepository,
    owner: &Object,
) -> Result<Vec<Object>, UseCaseError> {
    Ok(objects
        .all()?
        .into_iter()
        .filter(|o| o.base().parent.as_ref() == Some(owner.oid()))
        .collect())
}

pub(crate) fn holds_objects(
    objects: &dyn ObjectRepository,
    owner: &Oid,
) -> Result<bool, UseCaseError> {
    Ok(objects
        .all()?
        .iter()
        .any(|o| o.base().parent.as_ref() == Some(owner)))
}

pub(crate) fn is_held_by(
    objects: &dyn ObjectRepository,
    oid: &Oid,
    holder: &Oid,
) -> Result<bool, UseCaseError> {
    let mut seen = BTreeSet::new();
    let mut next = Some(oid.clone());
    while let Some(current) = next {
        if &current == holder {
            return Ok(true);
        }
        if !seen.insert(current.clone()) {
            return Ok(false);
        }
        next = objects.get(&current)?.and_then(|o| o.base().parent.clone());
    }
    Ok(false)
}

pub(crate) fn refuse_losing_a_held_folder(
    objects: &dyn ObjectRepository,
    before: &Object,
    after: &Object,
) -> Result<(), UseCaseError> {
    if loses_a_held_folder(objects, before, after)? {
        return Err(Refusal::SubjectNamesNoFolder(before.oid().clone()).into());
    }
    Ok(())
}

pub(crate) fn loses_a_held_folder(
    objects: &dyn ObjectRepository,
    before: &Object,
    after: &Object,
) -> Result<bool, UseCaseError> {
    Ok(
        match (before.base().owned_folder(), after.base().owned_folder()) {
            (Some(_), None) => holds_objects(objects, before.oid())?,
            _ => false,
        },
    )
}

pub(crate) fn move_into(
    objects: &dyn ObjectRepository,
    oid: &Oid,
    folder: &Path,
    parent: Option<Oid>,
) -> Result<(), UseCaseError> {
    let before = load(objects, oid)?;
    let mut after = before.clone();
    after.base_mut().path = folder.clone();
    after.base_mut().parent = parent;
    put_carrying_folder(objects, &before, &after)
}

pub(crate) fn replace_carrying_folder(
    objects: &dyn ObjectRepository,
    before: &Object,
    after: &Object,
) -> Result<(), UseCaseError> {
    let after = placed(objects, Some(before), after.clone())?;
    put_carrying_folder(objects, before, &after)
}

pub(crate) fn rebase_held(
    objects: &dyn ObjectRepository,
    owner: &Object,
    from: &Path,
    to: &Path,
) -> Result<(), UseCaseError> {
    for mut held in held_chain(objects, owner)? {
        if let Some(rebased) = held.base().path.rebase(from, to) {
            held.base_mut().path = rebased;
            objects.put(&held)?;
        }
    }
    Ok(())
}

fn put_carrying_folder(
    objects: &dyn ObjectRepository,
    before: &Object,
    after: &Object,
) -> Result<(), UseCaseError> {
    if let (Some(from), Some(to)) = (before.base().owned_folder(), after.base().owned_folder())
        && from != to
    {
        rebase_held(objects, before, &from, &to)?;
    }
    Ok(objects.put(after)?)
}

fn held_chain(objects: &dyn ObjectRepository, owner: &Object) -> Result<Vec<Object>, UseCaseError> {
    let all = objects.all()?;
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

#[cfg(test)]
mod tests;
