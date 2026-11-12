use dam_domain::{Object, Oid};

use crate::errors::UseCaseError;
use crate::ports::StoreError;
use crate::service::Dam;

impl Dam<'_> {
    pub fn seen_object(&self, oid: &Oid) -> Result<Option<Object>, UseCaseError> {
        match self.store.get(oid)? {
            Some(object) if self.access.sees(&object) => Ok(Some(self.as_seen(object)?)),
            _ => Ok(None),
        }
    }

    pub(crate) fn as_seen(&self, mut object: Object) -> Result<Object, StoreError> {
        if self.access.token().is_none() {
            return Ok(object);
        }
        let mut depends = Vec::new();
        for oid in std::mem::take(&mut object.base_mut().depends) {
            if !self.hides(&oid)? {
                depends.push(oid);
            }
        }
        object.base_mut().depends = depends;
        if let Object::Task(task) = &mut object
            && let Some(event) = &task.event
            && self.hides(event)?
        {
            task.event = None;
        }
        Ok(object)
    }

    fn hides(&self, oid: &Oid) -> Result<bool, StoreError> {
        Ok(!self.store.get(oid)?.is_some_and(|o| self.access.sees(&o)))
    }
}
