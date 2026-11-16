use super::*;
use crate::service::Dam;
use crate::testing::SlowHooks;

fn four_tasks() -> PullOutcome {
    PullOutcome {
        objects: (1..=4).map(|n| incoming("t", &format!("r{n}"))).collect(),
        sync: Some("s1".into()),
        ..PullOutcome::default()
    }
}

fn pulled_by(s: &Scenario, slow: &SlowHooks, outcome: PullOutcome) -> PullReport {
    let l = pulling(outcome);
    let dam = Dam {
        clock: slow,
        hooks: slow,
        ..s.dam()
    };
    dam.pull(remotes_launched_by(&l), None).unwrap().remove(0)
}

#[test]
fn the_hook_is_not_called_once_a_pull_has_run_it_for_sixty_seconds() {
    let s = pulled_hooked();
    let slow = SlowHooks::taking(&s.fixture.hooks, 30);
    let report = pulled_by(&s, &slow, four_tasks());
    assert_eq!(report.created, 4);
    let run = report.task_pulled.unwrap();
    assert_eq!((run.ran, run.not_reached), (2, 2));
    assert_eq!(calls(&s).len(), 2);
}
