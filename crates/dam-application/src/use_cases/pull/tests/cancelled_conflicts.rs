use super::cancellations::{meeting, run, sending_the_meeting_cancelled, tracked};
use crate::ports::Notice;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, remotes_launched_by};
use dam_domain::{Object, Task};

#[test]
fn a_cancelled_event_that_conflicts_still_raises_the_notice() {
    let s = tracked(meeting());
    let mut edited = meeting();
    edited.base.subject = "moved".into();
    s.committed(Object::Event(edited));
    let mut attached = Task::new(oid(6), "prep");
    attached.event = Some(oid(5));
    s.working(Object::Task(attached));
    let l = sending_the_meeting_cancelled();
    let reports = s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].conflicts, 1);
    assert_eq!(s.store.conflicts().unwrap().len(), 1);
    assert!(matches!(
        s.store.notices().unwrap().last(),
        Some(Notice::EventCancelled { attached: 1, .. })
    ));
}

#[test]
fn a_cancellation_for_an_event_with_no_snapshot_is_raised_against_the_working_copy() {
    let s = Scenario::default();
    s.committed(Object::Event(meeting()))
        .mapped_as(&oid(5), "e1");
    assert_eq!(run(&s, &["e1"]).unwrap()[0].conflicts, 1);
}
