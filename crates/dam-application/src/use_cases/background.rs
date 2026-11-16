use std::time::Duration;

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{BackgroundJob, BackgroundSync, Change, Invocation};
use crate::service::Dam;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundStep {
    Start,
    Stop,
    Enable,
    Disable,
}

impl BackgroundStep {
    pub fn verb(self) -> &'static str {
        match self {
            BackgroundStep::Start => "sync start",
            BackgroundStep::Stop => "sync stop",
            BackgroundStep::Enable => "sync enable",
            BackgroundStep::Disable => "sync disable",
        }
    }

    fn with_now(self) -> Option<BackgroundStep> {
        match self {
            BackgroundStep::Enable => Some(BackgroundStep::Start),
            BackgroundStep::Disable => Some(BackgroundStep::Stop),
            BackgroundStep::Start | BackgroundStep::Stop => None,
        }
    }

    fn begins_sync(self) -> bool {
        matches!(self, BackgroundStep::Start | BackgroundStep::Enable)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundRequest {
    pub remote: String,
    pub step: BackgroundStep,
    pub now: bool,
    pub interval: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundReport {
    pub remote: String,
    pub interval: Duration,
    pub done: Vec<(BackgroundStep, Change)>,
}

impl Dam<'_> {
    pub fn background_sync(
        &self,
        scheduler: &dyn BackgroundSync,
        invocation: &Invocation,
        request: &BackgroundRequest,
    ) -> Result<BackgroundReport, UseCaseError> {
        if self.access.token().is_some() {
            return Err(Refusal::NotOpenToTokens(request.step.verb()).into());
        }
        let remote = &request.remote;
        let configured = self
            .config
            .remote(remote)
            .ok_or_else(|| Refusal::NoSuchRemote(remote.clone()))?;
        if request.step.begins_sync() && !configured.sync.allowed() {
            return Err(Refusal::SyncNotAllowed(remote.clone()).into());
        }
        let steps =
            std::iter::once(request.step).chain(request.step.with_now().filter(|_| request.now));
        let job = BackgroundJob::new(invocation, remote, request.interval);
        let done = steps
            .map(|step| Ok((step, take(scheduler, &job, step)?)))
            .collect::<Result<_, UseCaseError>>()?;
        Ok(BackgroundReport {
            remote: remote.clone(),
            interval: request
                .interval
                .unwrap_or_else(|| configured.sync.interval()),
            done,
        })
    }
}

fn take(
    scheduler: &dyn BackgroundSync,
    job: &BackgroundJob,
    step: BackgroundStep,
) -> Result<Change, UseCaseError> {
    let change = match step {
        BackgroundStep::Start => scheduler.start(job),
        BackgroundStep::Stop => scheduler.stop(&job.remote),
        BackgroundStep::Enable => scheduler.enable(job),
        BackgroundStep::Disable => scheduler.disable(&job.remote),
    };
    Ok(change?)
}

#[cfg(test)]
mod tests;
