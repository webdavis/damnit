use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Field, Object, Path, Task};

use crate::errors::UseCaseError;
use crate::ports::{DoubtRepository, HelperError, Notice, NoticeRepository};
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{
    Pushed, Scenario, ScriptedHelper, ScriptedLauncher, accepting_all, oid, pushing,
    remotes_launched_by, task, task_caps, task_in, todoist,
};

fn timed_out() -> HelperError {
    HelperError::Timeout {
        helper: "todoist".into(),
        deadline: "2s".into(),
        said: None,
    }
}

fn timing_out_before_any_answer() -> (ScriptedLauncher, Pushed) {
    let handed = Pushed::default();
    let seen = handed.clone();
    let launcher = ScriptedLauncher::launching(move || {
        let seen = seen.clone();
        ScriptedHelper {
            push_answer: Box::new(move |ms| {
                seen.borrow_mut().extend(ms.iter().cloned());
                Err(timed_out())
            }),
            ..ScriptedHelper::answering(Ok(Default::default()))
        }
    });
    (launcher, handed)
}

fn push_timing_out(s: &Scenario) -> Pushed {
    let (l, handed) = timing_out_before_any_answer();
    let err = s.dam().push(remotes_launched_by(&l), None).unwrap_err();
    assert_eq!(err, UseCaseError::Helper(timed_out()));
    handed
}

#[test]
fn a_create_handed_over_before_a_timeout_is_in_doubt_and_nothing_else_is_stored() {
    let s = Scenario::default();
    s.committed(task(1));
    let handed = push_timing_out(&s);

    let every_field: BTreeSet<Field> = task_caps().fields.into_iter().collect();
    assert_eq!(
        s.store.fields_in_doubt(&todoist()).unwrap(),
        BTreeMap::from([(oid(1), every_field)])
    );
    assert!(s.store.notices().unwrap().is_empty());
    assert_eq!(s.store.last_push(&todoist()).unwrap(), None);
    assert_eq!(s.store.unpushed(&todoist()).unwrap().len(), 1);

    let (l, resent) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let resent = resent.borrow();
    assert_eq!(resent[0].op, MutationOp::Create);
    assert_eq!(
        resent[0].idempotency_key,
        handed.borrow()[0].idempotency_key
    );
}

#[test]
fn a_create_handed_over_before_a_timeout_then_removed_leaves_one_notice() {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "Ada", "Work/")));
    push_timing_out(&s);
    s.deleted(&oid(1));

    let (l, _) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.dam().push(remotes_launched_by(&l), None).unwrap();

    let removed = Notice::CreateInDoubtRemoved {
        remote: todoist(),
        oid: oid(1),
        subject: "Ada".into(),
        path: Path::parse("Work/").unwrap(),
    };
    let notices = s.store.notices().unwrap();
    assert_eq!(notices.iter().filter(|n| **n == removed).count(), 1);
}

#[test]
fn a_field_handed_over_before_a_timeout_is_sent_when_set_back() {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s.committed(Object::Task(Task::new(oid(1), "x")));
    push_timing_out(&s);
    s.committed(task(1));

    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].op, MutationOp::Update);
    assert_eq!(sent[0].fields, vec![Field::Subject]);
}
