use super::*;
use crate::config::HookEvent::{self, TaskNew};
use crate::errors::UseCaseError;
use crate::ports::{HelperError, Histories, Hooks, Pull};
use crate::service::Dam;
use crate::testing::{SlowHooks, oid};

#[test]
fn the_hook_is_called_for_no_more_tasks_after_sixty_seconds() {
    let s = every_hook();
    let slow = SlowHooks::taking(&s.fixture.hooks, 30);
    let dam = Dam {
        clock: &slow,
        hooks: &slow,
        ..s.dam()
    };
    let run = dam.run_hook_over_listed(TaskNew, None).unwrap();
    assert_eq!((run.ran, run.not_reached), (2, 1));
    assert_eq!(ran(&s).len(), 2);
}

struct PressingCtrlC<'a>(&'a Scenario);

impl Hooks for PressingCtrlC<'_> {
    fn run(
        &self,
        event: HookEvent,
        task: &Object,
        histories: &dyn Histories,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.0.fixture.interruption.pressed.set(true);
        self.0.fixture.hooks.run(event, task, histories)
    }

    fn pulled(
        &self,
        task: &Object,
        before: Option<&Object>,
        pull: &Pull,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.0.fixture.hooks.pulled(task, before, pull)
    }
}

#[test]
fn ctrl_c_stops_the_run_and_keeps_the_change_made() {
    let s = every_hook();
    requesting(&s, "buy oat milk");
    let pressing = PressingCtrlC(&s);
    let dam = Dam {
        hooks: &pressing,
        ..s.dam()
    };
    assert_eq!(
        dam.run_hook_over_listed(TaskNew, None),
        Err(UseCaseError::Helper(HelperError::Cancelled))
    );
    assert_eq!(ran(&s).len(), 1);
    assert_eq!(subject(&s, &oid(3)), "buy oat milk");
}
