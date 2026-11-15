use dam_domain::Oid;

use crate::ports::{AttemptRepository, Notice};
use crate::remote::{MutationOp, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, oid, pulling, pushing, remotes_launched_by, todoist,
};
use facts::{Family, Here, Owed, Row, every_row};

mod arrange;
mod facts;

type Sent = Vec<(MutationOp, Oid)>;

fn push_accepting_all(s: &Scenario) -> Sent {
    let (l, pushed) = pushing(accepting_all(Some("i:9")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let mut sent: Sent = pushed
        .borrow()
        .iter()
        .map(|m| (m.op, m.oid.clone()))
        .collect();
    sent.sort_by_key(|(_, oid)| oid.clone());
    sent
}

fn names(notice: &Notice, oid: &Oid) -> bool {
    match notice {
        Notice::RemovedUpstream { oid: o, .. } | Notice::PushFailed { oid: o, .. } => o == oid,
        _ => false,
    }
}

fn expected_first_push(row: Row) -> Sent {
    let mut sent = Vec::new();
    if row.edited_after {
        sent.push((MutationOp::Create, oid(1)));
    }
    if row.family == Family::ChildOfAdaDeletedAfter {
        sent.push((MutationOp::Delete, oid(3)));
    }
    sent
}

fn check(row: Row) {
    let s = arrange::before_the_removal(row);
    let removed = PullOutcome {
        removed: vec!["i:1".into()],
        ..PullOutcome::default()
    };
    let reports = s
        .dam()
        .pull(remotes_launched_by(&pulling(removed)), None)
        .unwrap();
    let noticed = row.here == Here::Kept && row.owed != Owed::DeleteUnanswered;
    assert_eq!(reports[0].removed_upstream, usize::from(noticed), "{row:?}");
    assert_eq!(
        reports[0].deletes_confirmed,
        usize::from(!noticed),
        "{row:?}"
    );
    let notices = s.store.notices().unwrap();
    let about_ann: Vec<&Notice> = notices.iter().filter(|n| names(n, &oid(1))).collect();
    let removed_upstream = matches!(about_ann[..], [Notice::RemovedUpstream { .. }]);
    assert!(
        about_ann.is_empty() != noticed && removed_upstream == noticed,
        "{row:?}: {about_ann:?}"
    );

    arrange::after_the_pull(&s, row);

    assert_eq!(push_accepting_all(&s), expected_first_push(row), "{row:?}");
    assert_eq!(push_accepting_all(&s), vec![], "{row:?}");
    let failed = s.store.notices().unwrap();
    let failed: Vec<&Notice> = failed
        .iter()
        .filter(|n| matches!(n, Notice::PushFailed { .. }))
        .collect();
    assert!(failed.is_empty(), "{row:?}: {failed:?}");
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty(), "{row:?}");
    assert!(s.store.attempts(&todoist()).unwrap().is_empty(), "{row:?}");
}

#[test]
fn every_way_a_removal_in_the_app_can_meet_what_dam_owes_ends_with_nothing_resent() {
    let rows = every_row();
    assert!(rows.len() > 30, "{}", rows.len());
    let failed: Vec<Row> = rows
        .into_iter()
        .filter(|row| std::panic::catch_unwind(|| check(*row)).is_err())
        .collect();
    assert!(
        failed.is_empty(),
        "{} rows failed: {failed:#?}",
        failed.len()
    );
}
