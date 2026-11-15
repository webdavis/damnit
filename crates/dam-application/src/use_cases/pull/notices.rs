use dam_domain::{EventStatus, Object};

use crate::errors::UseCaseError;
use crate::ports::{Notice, Store};

pub(super) fn note_cancelled(store: &dyn Store, object: &Object) -> Result<(), UseCaseError> {
    if let Object::Event(e) = object
        && e.status == EventStatus::Cancelled
    {
        let attached = store
            .all()?
            .iter()
            .filter(|o| {
                o.as_task()
                    .is_some_and(|t| t.event.as_ref() == Some(&e.base.oid))
            })
            .count();
        if attached > 0 {
            store.add_notice(&Notice::EventCancelled {
                oid: e.base.oid.clone(),
                subject: e.base.subject.clone(),
                attached,
            })?;
        }
    }
    Ok(())
}
