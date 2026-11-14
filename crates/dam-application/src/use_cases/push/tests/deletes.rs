use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, refusing_all, remotes_launched_by, task, todoist,
};

#[test]
fn a_successful_delete_clears_the_remote_mapping() {
    let s = Scenario::default();
    s.committed(task(1))
        .tracked_as(&oid(1), "r1")
        .pushed()
        .deleted(&oid(1));
    let (l, _) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_none());
    assert!(
        s.store
            .remote_snapshot(&todoist(), &oid(1))
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_delete_with_no_remote_mapping_is_skipped() {
    let s = Scenario::default();
    s.committed(task(1)).pushed().deleted(&oid(1));
    let (l, pushed) = pushing(|_| vec![]);
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].skipped, 1);
    assert_eq!(reports[0].sent, 0);
    assert!(pushed.borrow().is_empty());
}

#[test]
fn a_refused_delete_is_sent_again_on_the_next_push() {
    let s = Scenario::default();
    s.committed(task(1))
        .tracked_as(&oid(1), "r1")
        .pushed()
        .deleted(&oid(1));
    let (l, _) = pushing(refusing_all("temporary failure"));
    s.dam().push(remotes_launched_by(&l), None).unwrap();

    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent: Vec<_> = pushed
        .borrow()
        .iter()
        .map(|m| (m.oid.clone(), m.op))
        .collect();
    assert_eq!(sent, vec![(oid(1), MutationOp::Delete)]);
    assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_none());
}
