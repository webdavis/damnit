use super::PulledHookRun;
use crate::errors::UseCaseError;
use crate::ports::{FunctionFailure, Notice, RemoteName};
use crate::service::Dam;

impl Dam<'_> {
    pub(super) fn note_pulled_hook_run(
        &self,
        remote: &RemoteName,
        run: &PulledHookRun,
        error: Option<FunctionFailure>,
    ) -> Result<(), UseCaseError> {
        for held in self.store.notices()? {
            if matches!(&held, Notice::TaskPulledFailed { remote: noted, .. } if noted == remote) {
                self.store.remove_notice(&held)?;
            }
        }
        if run.failed > 0 || run.not_reached > 0 {
            self.store.add_notice(&Notice::TaskPulledFailed {
                remote: remote.clone(),
                failed: run.failed,
                not_reached: run.not_reached,
                error,
            })?;
        }
        Ok(())
    }
}
