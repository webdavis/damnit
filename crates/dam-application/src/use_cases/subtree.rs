use dam_domain::{Base, Object, Oid, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::use_cases::load::load;
use crate::use_cases::parents::{keep_parents_in_step, placed};

pub(crate) fn children(
    objects: &dyn ObjectRepository,
    owner: &Base,
) -> Result<Vec<Object>, UseCaseError> {
    let Some(folder) = owner.owned_folder() else {
        return Ok(Vec::new());
    };
    Ok(objects
        .all()?
        .into_iter()
        .filter(|o| o.base().path == folder)
        .collect())
}

pub(crate) fn holds_objects(
    objects: &dyn ObjectRepository,
    folder: &Path,
) -> Result<bool, UseCaseError> {
    Ok(objects
        .all()?
        .iter()
        .any(|o| o.base().path.is_within(folder)))
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
            (Some(folder), None) => holds_objects(objects, &folder)?,
            _ => false,
        },
    )
}

pub(crate) fn move_into(
    objects: &dyn ObjectRepository,
    oid: &Oid,
    folder: &Path,
) -> Result<(), UseCaseError> {
    let before = load(objects, oid)?;
    let mut after = before.clone();
    after.base_mut().path = folder.clone();
    replace_carrying_folder(objects, &before, &after)
}

pub(crate) fn replace_carrying_folder(
    objects: &dyn ObjectRepository,
    before: &Object,
    after: &Object,
) -> Result<(), UseCaseError> {
    let mut moved = Vec::new();
    if let (Some(from), Some(to)) = (before.base().owned_folder(), after.base().owned_folder())
        && from != to
    {
        moved.extend(move_folder(objects, &from, &to)?);
    }
    objects.put(&placed(objects, Some(before), after.clone())?)?;
    keep_parents_in_step(objects, &moved)
}

fn move_folder(
    objects: &dyn ObjectRepository,
    from: &Path,
    to: &Path,
) -> Result<Vec<Oid>, UseCaseError> {
    let mut moved = Vec::new();
    for mut object in objects.all()? {
        let Some(rebased) = object.base().path.rebase(from, to) else {
            continue;
        };
        object.base_mut().path = rebased;
        objects.put(&object)?;
        moved.push(object.oid().clone());
    }
    Ok(moved)
}

#[cfg(test)]
mod tests;
