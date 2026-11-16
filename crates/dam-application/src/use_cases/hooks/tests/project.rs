use dam_domain::Path;

use super::edits::hook_returns;
use super::{hooked, raised, retitled};
use crate::config::HookEvent::TaskChanged;
use crate::config::{Config, HookConfig, OnFailure};
use crate::errors::{Refusal, UseCaseError};
use crate::ports::HookFailure;
use crate::service::{Dam, ProjectHooks};
use crate::testing::{FakeHooks, Scenario, oid};

struct Project {
    config: Config,
    hooks: FakeHooks,
    within: Path,
}

fn project(within: &str, on_failure: Option<OnFailure>) -> Project {
    Project {
        config: Config {
            hooks: vec![HookConfig {
                event: TaskChanged,
                on_failure,
                file: ".damhooks/task_changed.lua".into(),
            }],
            ..crate::testing::config()
        },
        hooks: FakeHooks::default(),
        within: Path::parse(within).unwrap(),
    }
}

fn with<'a>(s: &'a Scenario, p: &'a Project) -> Dam<'a> {
    Dam {
        project_hooks: Some(ProjectHooks {
            config: &p.config,
            hooks: &p.hooks,
            within: &p.within,
        }),
        ..s.dam()
    }
}

#[test]
fn the_projects_hook_runs_after_the_persons_and_sees_the_task_as_it_left_it() {
    let s = hooked(&[(TaskChanged, None)]);
    let p = project("home/", None);
    hook_returns(&s, Ok(retitled("paid")));
    with(&s, &p)
        .edit(&oid(1), retitled("pay the rent"))
        .unwrap();
    assert_eq!(
        s.fixture.hooks.ran(),
        [(TaskChanged, "pay the rent".into())]
    );
    assert_eq!(p.hooks.ran(), [(TaskChanged, "paid".into())]);
}

#[test]
fn the_projects_hook_runs_with_no_personal_hook() {
    let s = hooked(&[]);
    let p = project("home/", None);
    with(&s, &p)
        .edit(&oid(1), retitled("pay the rent"))
        .unwrap();
    assert_eq!(p.hooks.ran(), [(TaskChanged, "pay the rent".into())]);
}

#[test]
fn the_projects_hook_never_runs_for_a_task_outside_the_project() {
    let s = hooked(&[(TaskChanged, None)]);
    let p = project("work/", None);
    with(&s, &p)
        .edit(&oid(1), retitled("pay the rent"))
        .unwrap();
    assert_eq!(s.fixture.hooks.ran().len(), 1);
    assert!(p.hooks.ran().is_empty());
}

#[test]
fn a_refusing_project_hook_refuses_the_command() {
    let s = hooked(&[(TaskChanged, None)]);
    let p = project("home/", Some(OnFailure::Refuse));
    *p.hooks.failing.borrow_mut() = Some(raised());
    assert_eq!(
        with(&s, &p).edit(&oid(1), retitled("pay the rent")),
        Err(UseCaseError::Refused(Refusal::HookRefused(HookFailure {
            event: TaskChanged,
            failure: raised(),
        })))
    );
}

#[test]
fn a_refusing_personal_hook_refuses_before_the_project_runs() {
    let s = hooked(&[(TaskChanged, Some(OnFailure::Refuse))]);
    let p = project("home/", None);
    *s.fixture.hooks.failing.borrow_mut() = Some(raised());
    assert!(
        with(&s, &p)
            .edit(&oid(1), retitled("pay the rent"))
            .is_err()
    );
    assert!(p.hooks.ran().is_empty());
}
