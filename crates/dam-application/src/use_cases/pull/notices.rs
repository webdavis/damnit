use dam_domain::{EventStatus, Object};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Notice, Store};

pub(super) fn record_removals(
    store: &dyn Store,
    remote: &RemoteConfig,
    removed: Vec<String>,
) -> Result<usize, UseCaseError> {
    let mut count = 0;
    for remote_id in removed {
        if let Some(oid) = store.oid_for_remote_id(&remote.name, &remote_id)? {
            let subject = store
                .get(&oid)?
                .map(|o| o.base().subject.clone())
                .unwrap_or_default();
            store.add_notice(&Notice::RemovedUpstream {
                remote: remote.name.clone(),
                oid: oid.clone(),
                subject,
            })?;
            store.clear_remote_mapping(&remote.name, &oid)?;
            count += 1;
        }
    }
    Ok(count)
}

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
