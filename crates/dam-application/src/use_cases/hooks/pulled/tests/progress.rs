use dam_domain::Permission::RemotePull;

use super::slow::SlowHooks;
use super::*;
use crate::access::Access;
use crate::service::Dam;
use crate::testing::{Said, allowed, holder};

fn four_tasks() -> PullOutcome {
    PullOutcome {
        objects: (1..=4).map(|n| incoming("t", &format!("r{n}"))).collect(),
        sync: Some("s1".into()),
        ..PullOutcome::default()
    }
}

fn pulled_by(s: &Scenario, access: &dyn Access) -> Vec<Said> {
    let l = pulling(four_tasks());
    let slow = SlowHooks::taking(&s.fixture.hooks, 0);
    let dam = Dam {
        clock: &slow,
        hooks: &slow,
        access,
        ..s.dam()
    };
    dam.pull(remotes_launched_by(&l), None).unwrap();
    s.fixture.progress.said.borrow().clone()
}

#[test]
fn the_hook_run_is_a_step_counted_task_by_task_that_finishes() {
    let s = pulled_hooked();
    let said = pulled_by(&s, &crate::access::Owner);
    assert_eq!(
        said,
        [
            Said::Started("running task_pulled tasks".into(), Some(4)),
            Said::Advanced(1),
            Said::Advanced(2),
            Said::Advanced(3),
            Said::Advanced(4),
            Said::Finished,
        ]
    );
}

#[test]
fn a_run_a_tokens_command_caused_shows_no_progress() {
    let s = pulled_hooked();
    let tool = holder(vec![allowed(&[RemotePull], None)]);
    assert_eq!(pulled_by(&s, &tool), []);
}
