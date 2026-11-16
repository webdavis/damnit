use dam_domain::Permission::{ConflictRead, EventRead, RemoteRead, TaskRead};
use dam_domain::{Object, Oid, Task};

use crate::access::Holder;
use crate::ports::{Conflict, MissingTask, Notice, NoticeRepository};
use crate::remote::RejectedObject;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid, task_in, todoist};
use crate::use_cases::status::Status;

fn work_and_home() -> Scenario {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.committed(Object::Task(task_in(2, "laundry", "home")));
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(2),
            ours: Some(Object::Task(task_in(2, "laundry", "home"))),
            theirs: Object::Task(Task::new(oid(2), "theirs")),
        })
        .unwrap();
    for n in [1, 2] {
        let failed = Notice::PushFailed {
            remote: todoist(),
            oid: oid(n),
            why: "the remote said no".into(),
        };
        s.store.add_notice(&failed).unwrap();
    }
    let notices = [
        Notice::RemovedUpstream {
            remote: todoist(),
            oid: oid(2),
            subject: "laundry".into(),
        },
        Notice::MissingUpstream {
            remote: todoist(),
            tasks: [(1, "report"), (2, "laundry")]
                .map(|(n, subject)| MissingTask {
                    oid: oid(n),
                    subject: subject.into(),
                })
                .to_vec(),
        },
        Notice::PullSkipped {
            remote: todoist(),
            object: RejectedObject {
                remote_id: "r9".into(),
                why: "unreadable".into(),
            },
        },
    ];
    for notice in notices {
        s.store.add_notice(&notice).unwrap();
    }
    s
}

fn status_for(s: &Scenario, paths: Option<&[&str]>) -> Status {
    let token: Holder = holder(vec![
        allowed(&[RemoteRead, ConflictRead], None),
        allowed(&[TaskRead, EventRead], paths),
    ]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    dam.status(&[todoist()]).unwrap()
}

fn named(notice: &Notice) -> Vec<Oid> {
    match notice {
        Notice::RemovedUpstream { oid, .. } | Notice::PushFailed { oid, .. } => vec![oid.clone()],
        Notice::MissingUpstream { tasks, .. } => tasks.iter().map(|t| t.oid.clone()).collect(),
        _ => vec![],
    }
}

#[test]
fn a_token_that_reads_nothing_sees_no_object_and_the_sentence() {
    let status = status_for(&work_and_home(), Some(&["nowhere"]));
    assert!(status.conflicts.is_empty());
    assert!(status.notices.is_empty(), "{:?}", status.notices);
    assert!(status.unpushed[0].oids.is_empty());
    assert!(status.unpushed[0].commit_ids.is_empty());
    assert!(status.unseen_failed);
}

#[test]
fn a_token_that_reads_part_sees_only_its_objects_and_no_remote_reason() {
    let status = status_for(&work_and_home(), Some(&["work"]));
    assert!(status.conflicts.is_empty());
    let shown: Vec<Oid> = status.notices.iter().flat_map(named).collect();
    assert_eq!(shown, [oid(1), oid(1)]);
    assert!(status.notices.iter().all(|n| !matches!(
        n,
        Notice::PushFailed { why, .. } if why == "the remote said no"
    )));
    assert_eq!(status.unpushed[0].oids, [oid(1)]);
    assert_eq!(status.unpushed[0].commit_ids.len(), 1);
    assert!(status.unseen_failed);
}

#[test]
fn a_token_that_reads_all_sees_every_object_and_no_sentence() {
    let status = status_for(&work_and_home(), None);
    assert_eq!(status.conflicts.len(), 1);
    assert_eq!(status.notices.iter().flat_map(named).count(), 5);
    assert!(status.notices.iter().any(|n| matches!(
        n,
        Notice::PushFailed { why, .. } if why == "the remote said no"
    )));
    assert_eq!(status.unpushed[0].oids.len(), 2);
    assert!(!status.unseen_failed);
}

#[test]
fn the_owner_sees_every_notice_as_stored_and_no_sentence() {
    let s = work_and_home();
    let status = s.dam().status(&[todoist()]).unwrap();
    assert_eq!(status.notices, s.store.notices().unwrap());
    assert_eq!(status.conflicts.len(), 1);
    assert!(!status.unseen_failed);
}

#[test]
fn a_conflict_whose_own_copy_is_hidden_is_not_shown_and_not_counted() {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(1),
            ours: Some(Object::Task(task_in(1, "report", "home"))),
            theirs: Object::Task(task_in(1, "report v2", "work")),
        })
        .unwrap();
    let status = status_for(&s, Some(&["work"]));
    assert!(status.conflicts.is_empty());
    assert!(status.unseen_failed);
}

#[test]
fn a_dropped_create_noted_at_a_place_the_token_cannot_read_is_not_shown() {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.store
        .add_notice(&Notice::UnansweredCreateDropped {
            remote: todoist(),
            oid: oid(1),
            subject: "report".into(),
            path: dam_domain::Path::parse("home").unwrap(),
            because: crate::ports::DroppedBecause::NoLongerSent,
        })
        .unwrap();
    let status = status_for(&s, Some(&["work"]));
    assert!(status.notices.is_empty(), "{:?}", status.notices);
    assert!(status.unseen_failed);
}

#[test]
fn a_cancelled_event_counts_only_the_attached_tasks_the_token_reads() {
    let s = Scenario::default();
    let mut meeting = dam_domain::Event::new(
        oid(5),
        "standup",
        dam_domain::When::Day(jiff::civil::date(2030, 1, 1)),
        dam_domain::When::Day(jiff::civil::date(2030, 1, 2)),
    );
    meeting.base.path = dam_domain::Path::parse("work").unwrap();
    s.committed(Object::Event(meeting));
    for (n, path) in [(6, "work"), (7, "home")] {
        let mut prep = task_in(n, "prep", path);
        prep.event = Some(oid(5));
        s.committed(Object::Task(prep));
    }
    let cancelled = Notice::EventCancelled {
        oid: oid(5),
        subject: "standup".into(),
        attached: 2,
    };
    s.store.add_notice(&cancelled).unwrap();
    let status = status_for(&s, Some(&["work"]));
    assert!(matches!(
        status.notices[..],
        [Notice::EventCancelled { attached: 1, .. }]
    ));
}
