use dam_domain::{Object, merge_fields};

use super::classify::Incoming;
use super::parents::{RemotePaths, as_seen};
use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Conflict, Store};
use crate::remote::RemoteCapabilities;
use crate::use_cases::last_seen::LastSeen;

pub(super) fn settled(
    store: &dyn Store,
    remote: &RemoteConfig,
    theirs: Object,
    remote_paths: &RemotePaths,
) -> Result<Incoming, UseCaseError> {
    let last_seen_on_the_remote = LastSeen::held(store, &remote.name, theirs.oid())?;
    Ok(
        if last_seen_on_the_remote == Some(as_seen(&theirs, remote_paths)) {
            Incoming::Unchanged
        } else {
            Incoming::Settled(theirs)
        },
    )
}

pub(super) fn against_local(
    store: &dyn Store,
    remote: &RemoteConfig,
    local: Object,
    theirs: Object,
    remote_paths: &RemotePaths,
) -> Result<Incoming, UseCaseError> {
    if local == theirs {
        return Ok(Incoming::Unchanged);
    }
    let oid = local.oid().clone();
    let last_seen_on_the_remote = LastSeen::held(store, &remote.name, &oid)?;
    if last_seen_on_the_remote.as_ref() == Some(&local) {
        return Ok(Incoming::FastForward(theirs));
    }
    if last_seen_on_the_remote == Some(as_seen(&theirs, remote_paths)) {
        return Ok(Incoming::Unchanged);
    }
    Ok(Incoming::Conflict(Box::new(Conflict {
        remote: remote.name.clone(),
        oid,
        ours: Some(local),
        theirs,
    })))
}

pub(super) fn changed_in_the_app(
    store: &dyn Store,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    theirs: &Object,
) -> Result<bool, UseCaseError> {
    let last_seen = LastSeen::held(store, &remote.name, theirs.oid())?;
    Ok(last_seen.is_none_or(|seen| merge_fields(&seen, theirs, &caps.fields) != seen))
}
