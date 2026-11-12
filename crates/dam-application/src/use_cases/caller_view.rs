use dam_domain::{Change, Kind, Object, Oid, Permission, Subject};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::StoreError;
use crate::service::Dam;
use crate::use_cases::load::{load, no_such_object};

impl Dam<'_> {
    pub fn seen_object(&self, oid: &Oid) -> Result<Option<Object>, UseCaseError> {
        match self.store.get(oid)? {
            Some(object) if self.access.sees(&object) => Ok(Some(self.as_seen(object)?)),
            _ => Ok(None),
        }
    }

    pub(crate) fn load_permitted(
        &self,
        oid: &Oid,
        permission: fn(Kind) -> Permission,
    ) -> Result<Object, UseCaseError> {
        let object = load(self.store, oid)?;
        let permission = permission(object.kind());
        if self.access.permits(permission, &[Subject::Object(&object)]) {
            Ok(object)
        } else if self.access.sees(&object) {
            Err(Refusal::NotPermitted(permission).into())
        } else {
            Err(no_such_object(oid).into())
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

    pub(crate) fn permits_change(&self, permission: Permission, change: &Change) -> bool {
        let touched: Vec<Subject<'_>> = change
            .before()
            .into_iter()
            .chain(change.after())
            .map(Subject::Object)
            .collect();
        self.access.permits(permission, &touched)
    }

    pub(crate) fn as_seen_change(&self, change: Change) -> Result<Change, StoreError> {
        Ok(match change {
            Change::Create { after } => Change::Create {
                after: self.as_seen(after)?,
            },
            Change::Update { before, after } => Change::Update {
                before: self.as_seen(before)?,
                after: self.as_seen(after)?,
            },
            Change::Delete { before } => Change::Delete {
                before: self.as_seen(before)?,
            },
        })
    }

    pub(crate) fn hides(&self, oid: &Oid) -> Result<bool, StoreError> {
        Ok(!self.store.get(oid)?.is_some_and(|o| self.access.sees(&o)))
    }
}
