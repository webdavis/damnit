use dam_domain::{Object, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;

pub(crate) fn load(objects: &dyn ObjectRepository, oid: &Oid) -> Result<Object, UseCaseError> {
    Ok(objects.get(oid)?.ok_or_else(|| no_such_object(oid))?)
}

pub(crate) fn no_such_object(oid: &Oid) -> Refusal {
    Refusal::NoSuchObject(oid.short().to_string())
}
