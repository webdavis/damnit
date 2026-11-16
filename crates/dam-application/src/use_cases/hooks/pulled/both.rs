use super::{PulledHookRun, PulledTask};
use crate::errors::UseCaseError;
use crate::ports::{FunctionFailure, HelperError, RemoteName};
use crate::service::Dam;

impl Dam<'_> {
    pub(crate) fn run_pulled_hook(
        &self,
        remote: &RemoteName,
        pulled: Vec<PulledTask>,
    ) -> Result<Option<PulledHookRun>, UseCaseError> {
        let mut ran = self.run_one_pulled_hook(remote, &pulled)?;
        if let Some(project) = self.project_hooks {
            let mut under = Vec::new();
            for task in pulled {
                if let Some(now) = self.store.get(&task.oid)?
                    && now.base().path.is_within(project.within)
                {
                    under.push(task);
                }
            }
            let as_project = Dam {
                config: project.config,
                hooks: project.hooks,
                project_hooks: None,
                ..*self
            };
            ran = joined(ran, as_project.run_one_pulled_hook(remote, &under)?);
        }
        let Some((run, first_error)) = ran else {
            return Ok(None);
        };
        self.note_pulled_hook_run(remote, &run, first_error)?;
        if self.interruption.requested() {
            return Err(HelperError::Cancelled.into());
        }
        Ok(Some(run))
    }
}

type Ran = Option<(PulledHookRun, Option<FunctionFailure>)>;

fn joined(person: Ran, project: Ran) -> Ran {
    match (person, project) {
        (Some((first, first_error)), Some((then, then_error))) => Some((
            PulledHookRun {
                ran: first.ran + then.ran,
                changed: first.changed + then.changed,
                failed: first.failed + then.failed,
                not_reached: first.not_reached + then.not_reached,
                commit: then.commit.or(first.commit),
            },
            first_error.or(then_error),
        )),
        (one, other) => one.or(other),
    }
}
