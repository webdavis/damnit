use super::unanswered::{named, pulled, push_with_no_answer};
use crate::remote::PullOutcome;
use crate::testing::{Scenario, oid, pulling, remotes_launched_by};

fn values_kept_by_a_pull_of(s: &Scenario, subject: &str) -> usize {
    let outcome = PullOutcome {
        objects: vec![pulled(named(subject), "i:1")],
        ..PullOutcome::default()
    };
    let reports = s.dam().pull(remotes_launched_by(&pulling(outcome)), None);
    reports.unwrap()[0].values_kept
}

#[test]
fn every_pull_counts_a_value_kept_while_its_write_is_unanswered() {
    let s = Scenario::default();
    s.committed(named("a")).tracked_as(&oid(1), "i:1").pushed();
    s.committed(named("b"));
    push_with_no_answer(&s);
    s.committed(named("a"));

    assert_eq!(values_kept_by_a_pull_of(&s, "b"), 1);
    assert_eq!(values_kept_by_a_pull_of(&s, "b"), 1);
}
