use super::edits::requesting;
use super::*;
use crate::errors::UseCaseError;
use crate::ports::{
    CommitRepository, EditRequest, FunctionFailure, HelperError, Hooks, Notice, NoticeRepository,
    Pull,
};
use crate::service::Dam;

struct PressingCtrlC<'a>(&'a Scenario);

impl Hooks for PressingCtrlC<'_> {
    fn run(
        &self,
        event: crate::config::HookEvent,
        task: &Object,
        histories: &dyn crate::ports::Histories,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.0.fixture.hooks.run(event, task, histories)
    }

    fn pulled(
        &self,
        task: &Object,
        before: Option<&Object>,
        pull: &Pull,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.0.fixture.interruption.pressed.set(true);
        self.0.fixture.hooks.pulled(task, before, pull)
    }
}

#[test]
fn ctrl_c_stops_the_hook_keeps_the_pull_and_commits_what_it_changed() {
    let s = pulled_hooked();
    requesting(&s, "filed");
    let l = pulling(PullOutcome {
        objects: vec![
            incoming("a", "r1"),
            incoming("b", "r2"),
            incoming("c", "r3"),
        ],
        sync: Some("s1".into()),
        ..PullOutcome::default()
    });
    let pressing = PressingCtrlC(&s);
    let dam = Dam {
        hooks: &pressing,
        ..s.dam()
    };
    let answer = dam.pull(remotes_launched_by(&l), None);
    assert!(matches!(
        answer,
        Err(UseCaseError::Helper(HelperError::Cancelled))
    ));
    assert_eq!(calls(&s).len(), 1);
    assert_eq!(s.store.all().unwrap().len(), 3);
    let log = s.store.log().unwrap();
    assert_eq!(log[0].message, "task_pulled after pull from todoist");
    assert_eq!(log.len(), 2);
    assert_eq!(
        s.store.notices().unwrap(),
        [Notice::TaskPulledFailed {
            remote: todoist(),
            failed: 0,
            not_reached: 2,
            error: None,
        }]
    );
}
