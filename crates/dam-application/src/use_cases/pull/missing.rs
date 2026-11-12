use std::collections::HashSet;

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{MissingTask, Notice, Store};

pub(super) fn record_missing(
    store: &dyn Store,
    remote: &RemoteConfig,
    listed: &HashSet<String>,
) -> Result<(), UseCaseError> {
    for held in store.notices()? {
        if matches!(&held, Notice::MissingUpstream { remote: r, .. } if *r == remote.name) {
            store.remove_notice(&held)?;
        }
    }
    let mut tasks = Vec::new();
    for object in store.all()? {
        let oid = &object.base().oid;
        if !object.as_task().is_some_and(|t| !t.completion.is_done()) {
            continue;
        }
        let Some(remote_id) = store.remote_id(&remote.name, oid)? else {
            continue;
        };
        let unchanged_since_last_seen =
            store.remote_snapshot(&remote.name, oid)?.as_ref() == Some(&object);
        if unchanged_since_last_seen && !listed.contains(&remote_id) {
            tasks.push(MissingTask {
                oid: oid.clone(),
                subject: object.base().subject.clone(),
            });
        }
    }
    if !tasks.is_empty() {
        store.add_notice(&Notice::MissingUpstream {
            remote: remote.name.clone(),
            tasks,
        })?;
    }
    Ok(())
}
