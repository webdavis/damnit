use crate::testing::prelude::*;
use crate::testing::{Scenario, accepting_all, oid, pushing, remotes_launched_by, task, todoist};

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
