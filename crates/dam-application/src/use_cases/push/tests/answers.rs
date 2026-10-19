use crate::ports::Notice;
use crate::remote::{MutationOutcome, MutationVerdict};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, refusing_all, remotes_launched_by, task, todoist,
};

#[test]
fn a_failed_mutation_becomes_a_notice_and_a_retry() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(refusing_all("rate limited"));
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        reports[0].failed,
        vec![(oid(1), "rate limited".to_string())]
    );
    assert_eq!(s.store.push_retries(&todoist()).unwrap(), vec![oid(1)]);
    assert!(matches!(
        s.store.notices().unwrap()[0],
        Notice::PushFailed { .. }
    ));
    assert!(
        s.store.unpushed(&todoist()).unwrap().is_empty(),
        "the commit is marked pushed; the retry set carries the failure forward"
    );
}

#[test]
fn a_mutation_the_helper_never_answered_is_a_failure() {
    let s = Scenario::default();
    s.committed(task(1)).committed(task(2));
    let accept = accepting_all(Some("r1"));
    let (l, _) = pushing(move |ms| {
        accept(ms)
            .into_iter()
            .filter(|answer| answer.oid == oid(1))
            .collect()
    });
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    assert_eq!(
        reports[0].failed,
        vec![(oid(2), "no answer from helper".to_string())]
    );
    assert_eq!(s.store.push_retries(&todoist()).unwrap(), vec![oid(2)]);
    assert!(matches!(
        s.store.notices().unwrap()[0],
        Notice::PushFailed { .. }
    ));
    let still_unpushed = s.store.unpushed(&todoist()).unwrap();
    assert_eq!(still_unpushed.len(), 1);
    assert!(still_unpushed[0].changes.iter().any(|c| *c.oid() == oid(2)));
}

#[test]
fn duplicate_and_unsent_results_are_ignored() {
    let s = Scenario::default();
    s.committed(task(1));
    let refused = |byte: u8, why: &str| MutationOutcome {
        oid: oid(byte),
        verdict: MutationVerdict::Refused { why: why.into() },
    };
    let (l, _) = pushing(move |_| {
        vec![
            refused(1, "first"),
            refused(1, "second"),
            refused(9, "unsent"),
        ]
    });
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].succeeded, 0);
    assert_eq!(reports[0].failed.len(), 1);
    let retries = s.store.push_retries(&todoist()).unwrap();
    assert_eq!(retries, vec![oid(1)]);
    assert!(!retries.contains(&oid(9)));
}
