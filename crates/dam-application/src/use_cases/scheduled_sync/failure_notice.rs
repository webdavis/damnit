use crate::errors::UseCaseError;
use crate::ports::{Notice, RemoteName};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;

impl Dam<'_> {
    pub(super) fn replace_sync_failure(
        &self,
        remote: &RemoteName,
        why: &str,
    ) -> Result<(), UseCaseError> {
        atomically(self.store, || {
            self.clear_sync_failure(remote)?;
            Ok(self.store.add_notice(&Notice::SyncFailed {
                remote: remote.clone(),
                why: why.to_string(),
            })?)
        })
    }

    pub(super) fn clear_sync_failure(&self, remote: &RemoteName) -> Result<(), UseCaseError> {
        let failures =
            self.store.notices()?.into_iter().filter(
                |n| matches!(n, Notice::SyncFailed { remote: failed, .. } if failed == remote),
            );
        for failure in failures {
            self.store.remove_notice(&failure)?;
        }
        Ok(())
    }
}
