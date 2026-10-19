mod accepted_kinds;
mod answers;
mod deletes;
mod keys;
mod last_push;
mod read_only;
mod reconcile;
mod refusals;
mod retries;
mod verbs;

use crate::remote::MutationOp;
use dam_domain::{Event, Object, When};
use jiff::civil::date;

use crate::testing::prelude::*;
use crate::testing::{Scenario, accepting_all, oid, pushing, remotes_launched_by, task, todoist};

#[test]
fn a_committed_create_is_sent_and_its_remote_id_is_mapped() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(accepting_all(Some("r1")));
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    assert_eq!(pushed.borrow()[0].op, MutationOp::Create);
    assert_eq!(
        s.store.remote_id(&todoist(), &oid(1)).unwrap().as_deref(),
        Some("r1")
    );
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
    assert_eq!(
        *l.launched_with.borrow(),
        vec![vec![("api_token".to_string(), "t".into())]],
        "one helper process, launched with the token the remote's config declares"
    );
}

#[test]
fn a_retry_is_sent_as_an_update_of_the_committed_state() {
    let s = Scenario::default();
    s.committed(task(1));
    s.store.map_remote_id(&todoist(), &oid(1), "r1").unwrap();
    s.store.set_push_retries(&todoist(), &[oid(1)]).unwrap();
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert!(
        sent.iter()
            .any(|m| m.op == MutationOp::Update && m.remote_id.as_deref() == Some("r1"))
    );
    assert!(s.store.push_retries(&todoist()).unwrap().is_empty());
}

#[test]
fn an_event_is_skipped_by_a_task_only_helper() {
    let s = Scenario::default();
    s.committed(Object::Event(Event::new(
        oid(2),
        "e",
        When::Day(date(2026, 1, 1)),
        When::Day(date(2026, 1, 2)),
    )));
    let (l, pushed) = pushing(|_| vec![]);
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].skipped, 1);
    assert!(pushed.borrow().is_empty());
}
