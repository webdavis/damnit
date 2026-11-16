use dam_domain::{Object, Oid, nearest_parents};

use crate::errors::UseCaseError;
use crate::ports::ObjectRepository;

pub(crate) fn with_nearest_parent(
    objects: &dyn ObjectRepository,
    mut object: Object,
) -> Result<Object, UseCaseError> {
    let all = objects.all()?;
    let parents = nearest_parents(&object.base().path, &all);
    object.base_mut().parent = match <[Oid; 1]>::try_from(parents) {
        Ok([parent]) => Some(parent),
        Err(_) => None,
    };
    Ok(object)
}

pub(crate) fn placed(
    objects: &dyn ObjectRepository,
    before: Option<&Object>,
    mut after: Object,
) -> Result<Object, UseCaseError> {
    let kept = before
        .filter(|before| before.base().path == after.base().path)
        .and_then(|before| after.base().parent.clone().or(before.base().parent.clone()));
    if let Some(parent) = kept
        && objects.get(&parent)?.is_some()
    {
        after.base_mut().parent = Some(parent);
        return Ok(after);
    }
    with_nearest_parent(objects, after)
}

pub(crate) fn put_earlier_copy(
    objects: &dyn ObjectRepository,
    copy: &Object,
) -> Result<(), UseCaseError> {
    let gone = match &copy.base().parent {
        Some(parent) => objects.get(parent)?.is_none(),
        None => false,
    };
    if gone {
        return Ok(objects.put(&with_nearest_parent(objects, copy.clone())?)?);
    }
    Ok(objects.put(copy)?)
}

pub(crate) fn keep_parents_in_step(
    objects: &dyn ObjectRepository,
    oids: &[Oid],
) -> Result<(), UseCaseError> {
    for oid in oids {
        if let Some(object) = objects.get(oid)? {
            objects.put(&with_nearest_parent(objects, object)?)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
