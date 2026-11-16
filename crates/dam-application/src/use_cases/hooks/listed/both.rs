use dam_domain::{Object, Oid};

use super::ListedHookRun;
use crate::config::HookEvent;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;

impl Dam<'_> {
    pub fn run_hook_over_listed(
        &self,
        hook: HookEvent,
        query: Option<&str>,
    ) -> Result<ListedHookRun, UseCaseError> {
        let project = self.project_hooks.filter(|p| p.config.declares(hook));
        let personal = self.config.declares(hook);
        if !personal && project.is_none() {
            return Err(Refusal::HookNotDeclared(hook).into());
        }
        let listed: Vec<Oid> = self
            .list(query)?
            .into_iter()
            .filter_map(|object| match object {
                Object::Task(task) => Some(task.base.oid),
                Object::Event(_) => None,
            })
            .collect();
        let mut run = None;
        if personal {
            run = Some(self.run_one_hook_over(hook, &listed)?);
        }
        let Some(project) = project else {
            return Ok(run.unwrap_or_else(|| nothing_ran(hook)));
        };
        let mut under = Vec::new();
        for oid in listed {
            if let Some(now) = self.store.get(&oid)?
                && now.base().path.is_within(project.within)
            {
                under.push(oid);
            }
        }
        let as_project = Dam {
            config: project.config,
            hooks: project.hooks,
            project_hooks: None,
            ..*self
        };
        let theirs = as_project.run_one_hook_over(hook, &under)?;
        Ok(match run {
            Some(first) => joined(first, theirs),
            None => theirs,
        })
    }
}

fn nothing_ran(hook: HookEvent) -> ListedHookRun {
    ListedHookRun {
        hook,
        ran: 0,
        changed: 0,
        failed: 0,
        not_reached: 0,
        error: None,
    }
}

fn joined(first: ListedHookRun, then: ListedHookRun) -> ListedHookRun {
    ListedHookRun {
        hook: first.hook,
        ran: first.ran + then.ran,
        changed: first.changed + then.changed,
        failed: first.failed + then.failed,
        not_reached: first.not_reached + then.not_reached,
        error: first.error.or(then.error),
    }
}
