use dam_domain::Permission::RemotePull;

use super::slow::SlowHooks;
use super::*;
use crate::access::Access;
use crate::config::HookEvent;
use crate::service::Dam;
use crate::testing::{allowed, holder};

fn four_tasks() -> PullOutcome {
    PullOutcome {
        objects: (1..=4).map(|n| incoming("t", &format!("r{n}"))).collect(),
        sync: Some("s1".into()),
        ..PullOutcome::default()
    }
}

fn pulled_by(s: &Scenario, slow: &SlowHooks, access: &dyn Access) {
    let l = pulling(four_tasks());
    let dam = Dam {
        clock: slow,
        hooks: slow,
        access,
        ..s.dam()
    };
    dam.pull(remotes_launched_by(&l), None).unwrap();
}

fn said(s: &Scenario) -> Vec<(HookEvent, usize)> {
    s.fixture.hook_progress.said.borrow().clone()
}

#[test]
fn after_two_seconds_it_says_once_how_many_tasks_the_hook_runs_for() {
    let s = pulled_hooked();
    pulled_by(
        &s,
        &SlowHooks::taking(&s.fixture.hooks, 1),
        &crate::access::Owner,
    );
    assert_eq!(said(&s), [(HookEvent::TaskPulled, 4)]);
}

#[test]
fn a_quick_run_says_nothing() {
    let s = pulled_hooked();
    pulled_by(
        &s,
        &SlowHooks::taking(&s.fixture.hooks, 0),
        &crate::access::Owner,
    );
    assert_eq!(said(&s), []);
}

#[test]
fn a_run_a_tokens_command_caused_says_nothing() {
    let s = pulled_hooked();
    let tool = holder(vec![allowed(&[RemotePull], None)]);
    pulled_by(&s, &SlowHooks::taking(&s.fixture.hooks, 1), &tool);
    assert_eq!(said(&s), []);
}
