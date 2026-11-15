use dam_domain::{Object, Oid};

use super::order::held_upstream;
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, remotes_launched_by, task_in, todoist,
};

use MutationOp::{Create, Update};

type Sent = Vec<(Oid, MutationOp, Option<String>)>;

fn named(byte: u8, subject: &str) -> Object {
    Object::Task(task_in(byte, subject, "Home/"))
}

fn withholding(stuck: Oid) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static {
    move |ms| {
        let accept = accepting_all(Some("i:new"));
        accept(ms)
            .into_iter()
            .map(|mut outcome| {
                if outcome.oid == stuck {
                    outcome.verdict = MutationVerdict::Unanswered {
                        why: "no answer".into(),
                    };
                }
                outcome
            })
            .collect()
    }
}

fn push_with(
    s: &Scenario,
    answer: impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static,
) -> Sent {
    let (l, pushed) = pushing(answer);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    pushed
        .borrow()
        .iter()
        .map(|m| {
            let subject = m.object.as_ref().map(|o| o.base().subject.clone());
            (m.oid.clone(), m.op, subject)
        })
        .collect()
}

fn subject(of: Option<Object>) -> Option<String> {
    of.map(|o| o.base().subject.clone())
}

fn assert_all_agree(s: &Scenario, byte: u8, remote_holds: &str) {
    let committed = subject(s.store.committed(&oid(byte)).unwrap());
    let last_seen = subject(s.store.remote_snapshot(&todoist(), &oid(byte)).unwrap());
    assert_eq!(committed.as_deref(), Some(remote_holds));
    assert_eq!(last_seen.as_deref(), Some(remote_holds));
}

fn renamed_together_with_t2_withheld() -> Scenario {
    let s = held_upstream(&[task_in(1, "t1", "Home/"), task_in(2, "t2", "Home/")]);
    s.working(named(1, "x1")).committed(named(2, "x2"));
    assert_eq!(
        push_with(&s, withholding(oid(2))),
        vec![
            (oid(1), Update, Some("x1".into())),
            (oid(2), Update, Some("x2".into())),
        ]
    );
    s
}

#[test]
fn a_later_push_never_resends_an_older_value_from_a_partly_answered_commit() {
    let s = renamed_together_with_t2_withheld();
    s.committed(named(1, "x1 edited"));
    assert_eq!(
        push_with(&s, withholding(oid(2))),
        vec![
            (oid(1), Update, Some("x1 edited".into())),
            (oid(2), Update, Some("x2".into())),
        ]
    );
    assert_eq!(
        push_with(&s, accepting_all(Some("i:new"))),
        vec![(oid(2), Update, Some("x2".into()))]
    );
    assert_all_agree(&s, 1, "x1 edited");
    assert_all_agree(&s, 2, "x2");
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
    assert!(push_with(&s, accepting_all(Some("i:new"))).is_empty());
}

#[test]
fn a_later_push_never_resends_the_delete_of_a_task_created_again() {
    let s = held_upstream(&[task_in(1, "t1", "Home/"), task_in(2, "t2", "Home/")]);
    s.store.delete(&oid(1)).unwrap();
    s.committed(named(2, "x2"));
    push_with(&s, withholding(oid(2)));
    s.committed(named(1, "t1 again"));
    assert_eq!(
        push_with(&s, withholding(oid(2))),
        vec![
            (oid(1), Create, Some("t1 again".into())),
            (oid(2), Update, Some("x2".into())),
        ]
    );
    assert_eq!(
        push_with(&s, accepting_all(Some("i:new"))),
        vec![(oid(2), Update, Some("x2".into()))]
    );
    assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_some());
    assert_all_agree(&s, 1, "t1 again");
    assert_all_agree(&s, 2, "x2");
}
