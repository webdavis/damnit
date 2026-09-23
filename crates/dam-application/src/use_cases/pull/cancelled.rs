//! A cancellation the remote reported by id alone, as the whole object the
//! rest of the pull already knows how to land.

use dam_domain::{EventStatus, Object};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::Repositories;
use crate::remote::IncomingObject;

/// Each remote id the remote cancelled, as the event dam last knew under it
/// with its status set to cancelled. An id dam never mapped, and one mapped
/// to a task, yields nothing.
///
/// The base is the remote's own last snapshot, which is what the remote
/// held before it cancelled; the working copy stands in when no snapshot was
/// kept. Either way only the status differs, so classification sees exactly
/// one upstream change.
pub(super) fn cancelled_events(
    repos: Repositories<'_>,
    remote: &RemoteConfig,
    cancelled: &[String],
) -> Result<Vec<IncomingObject>, UseCaseError> {
    let mut out = Vec::new();
    for remote_id in cancelled {
        let Some(oid) = repos
            .remote_tracking
            .oid_for_remote_id(&remote.name, remote_id)?
        else {
            continue;
        };
        let known = match repos.remote_tracking.remote_snapshot(&remote.name, &oid)? {
            Some(snapshot) => Some(snapshot),
            None => repos.objects.get(&oid)?,
        };
        if let Some(Object::Event(mut event)) = known {
            event.status = EventStatus::Cancelled;
            out.push(IncomingObject {
                remote_id: remote_id.clone(),
                object: Object::Event(event),
            });
        }
    }
    Ok(out)
}
