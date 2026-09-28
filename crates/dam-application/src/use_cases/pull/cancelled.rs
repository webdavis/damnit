use dam_domain::{EventStatus, Object};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::Repositories;
use crate::remote::IncomingObject;

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
        let snapshot_or_working_copy =
            match repos.remote_tracking.remote_snapshot(&remote.name, &oid)? {
                Some(snapshot) => Some(snapshot),
                None => repos.objects.get(&oid)?,
            };
        if let Some(Object::Event(mut event)) = snapshot_or_working_copy {
            event.status = EventStatus::Cancelled;
            out.push(IncomingObject {
                remote_id: remote_id.clone(),
                object: Object::Event(event),
            });
        }
    }
    Ok(out)
}
