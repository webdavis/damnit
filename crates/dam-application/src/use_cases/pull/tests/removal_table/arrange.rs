use dam_domain::{Object, Task};

use super::facts::{Family, Here, Owed, Row};
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pushing, refusing_all, remotes_launched_by, task_in};
use crate::use_cases::pull::tests::unanswered::{push_answering_creates_only, push_with_no_answer};

fn ann(row: Row, body: &str) -> Object {
    let path = match row.family {
        Family::ChildOfAdaDeletedAfter => "work/Ada/",
        _ => "work/",
    };
    let mut ann = task_in(1, "Ann", path);
    ann.base.body = body.into();
    Object::Task(ann)
}

fn tracked(s: &Scenario, task: Task, remote_id: &str) {
    let oid = task.base.oid.clone();
    s.committed(Object::Task(task))
        .tracked_as(&oid, remote_id)
        .pushed();
}

fn push_refused(s: &Scenario) {
    let (l, _) = pushing(refusing_all("no"));
    let _ = s.dam().push(remotes_launched_by(&l), None);
}

pub(super) fn before_the_removal(row: Row) -> Scenario {
    let s = Scenario::default().drawing_from(42);
    match row.family {
        Family::ParentOfBob => tracked(&s, task_in(2, "Bob", "work/Ann/"), "i:2"),
        Family::ChildOfAdaDeletedAfter => tracked(&s, task_in(3, "Ada", "work/"), "i:3"),
        Family::Alone => {}
    }
    if row.owed == Owed::CreateAcceptedFollowUpUnanswered {
        s.committed(ann(row, "first"));
        push_with_no_answer(&s);
        s.committed(ann(row, "second"));
        push_answering_creates_only(&s, "i:1");
    } else {
        let Object::Task(first) = ann(row, "first") else {
            unreachable!()
        };
        tracked(&s, first, "i:1");
    }
    match row.owed {
        Owed::Nothing | Owed::CreateAcceptedFollowUpUnanswered => {}
        Owed::CommittedNotPushed => {
            s.committed(ann(row, "second"));
        }
        Owed::Unanswered => {
            s.committed(ann(row, "second"));
            push_with_no_answer(&s);
        }
        Owed::Refused => {
            s.committed(ann(row, "second"));
            push_refused(&s);
        }
        Owed::DeleteUnanswered => {
            s.deleted(&oid(1));
            push_with_no_answer(&s);
        }
    }
    match row.here {
        Here::Kept => {}
        Here::RemovedNotCommitted => s.store.delete(&oid(1)).unwrap(),
        Here::RemovedCommitted if row.owed == Owed::DeleteUnanswered => {}
        Here::RemovedCommitted => {
            s.deleted(&oid(1));
        }
    }
    s
}

pub(super) fn after_the_pull(s: &Scenario, row: Row) {
    if row.edited_after {
        s.committed(ann(row, "after"));
    }
    if row.family == Family::ChildOfAdaDeletedAfter {
        if s.store.get(&oid(1)).unwrap().is_some() {
            s.store.delete(&oid(1)).unwrap();
        }
        s.deleted(&oid(3));
    }
}
