use super::incoming;
use crate::ports::Notice;
use crate::remote::{PullOutcome, RejectedObject};
use crate::testing::prelude::*;
use crate::testing::{Scenario, pulling, remotes_launched_by, todoist};

#[test]
fn a_rejection_with_no_remote_id_is_noticed_as_skipped_by_its_reason_alone() {
    let s = Scenario::default();
    let l = pulling(PullOutcome {
        rejected: vec![RejectedObject {
            remote_id: String::new(),
            why: "an object with no remote id was skipped".into(),
        }],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        s.store.notices().unwrap(),
        vec![Notice::PullSkipped {
            remote: todoist(),
            object: RejectedObject {
                remote_id: String::new(),
                why: "an object with no remote id was skipped".into(),
            },
        }]
    );
}

#[test]
fn one_unconvertible_object_is_skipped_with_a_notice_and_the_rest_still_pull() {
    let s = Scenario::default();
    let l = pulling(PullOutcome {
        objects: vec![incoming("from todoist", "r-good")],
        rejected: vec![RejectedObject {
            remote_id: "r-bad".into(),
            why: "path: cannot read \"a//b/\"".into(),
        }],
        sync: Some("s1".into()),
        ..PullOutcome::default()
    });
    let reports = s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].created, 1);
    assert_eq!(
        s.store.notices().unwrap(),
        vec![Notice::PullSkipped {
            remote: todoist(),
            object: RejectedObject {
                remote_id: "r-bad".into(),
                why: "path: cannot read \"a//b/\"".into(),
            },
        }]
    );
    assert!(
        s.store
            .oid_for_remote_id(&todoist(), "r-bad")
            .unwrap()
            .is_none()
    );
}
