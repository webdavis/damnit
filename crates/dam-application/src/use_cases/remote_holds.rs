use dam_domain::Oid;

use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{RemoteTrackingRepository, StoreError};
use crate::service::Dam;

pub fn held_by_a_remote(
    tracking: &dyn RemoteTrackingRepository,
    remotes: &[RemoteConfig],
    oid: &Oid,
) -> Result<bool, StoreError> {
    for remote in remotes {
        if tracking.remote_id(&remote.name, oid)?.is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

impl Dam<'_> {
    pub fn telling_whether_a_remote_holds(&self, error: UseCaseError) -> UseCaseError {
        let UseCaseError::Refused(Refusal::ParentNotKnown {
            oid,
            unsettled,
            namesakes,
            folder,
            ..
        }) = error
        else {
            return error;
        };
        let held_by_a_remote = unsettled
            .iter()
            .any(|u| held_by_a_remote(self.store, &self.config.remotes, u).unwrap_or(false));
        Refusal::ParentNotKnown {
            oid,
            unsettled,
            namesakes,
            folder,
            held_by_a_remote,
        }
        .into()
    }
}

#[cfg(test)]
mod tests;
