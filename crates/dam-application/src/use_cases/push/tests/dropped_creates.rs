use dam_domain::{Object, Path};

use crate::ports::{AttemptRepository, DroppedBecause, HelperError, Notice, NoticeRepository};
use crate::remote::{MutationOp, PullOutcome};
use crate::testing::{
    Scenario, ScriptedHelper, ScriptedLauncher, accepting_all, config, oid, pulling, pushing,
    refusing_all, remotes_launched_by, task_in, todoist,
};

fn scoped_to_work() -> Scenario {
    let mut config = config();
    config.remotes[0].path = Some(Path::parse("Work").unwrap());
    Scenario::default().configured(config)
}

fn ada_in(path: &str) -> Object {
    Object::Task(task_in(1, "Ada", path))
}

fn push_with_no_answer(s: &Scenario) {
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
}

fn push_accepting(s: &Scenario) -> Vec<MutationOp> {
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    pushed.borrow().iter().map(|m| m.op).collect()
}

fn dropped(s: &Scenario, because: DroppedBecause) -> usize {
    let notice = Notice::UnansweredCreateDropped {
        remote: todoist(),
        oid: oid(1),
        subject: "Ada".into(),
        path: Path::parse("Work/").unwrap(),
        because,
    };
    let notices = s.store.notices().unwrap();
    notices.iter().filter(|n| **n == notice).count()
}

fn attempted(s: &Scenario) -> bool {
    s.store.attempts(&todoist()).unwrap().contains_key(&oid(1))
}

#[test]
fn an_unanswered_create_removed_before_its_resend_is_noticed_once_and_forgotten() {
    let s = Scenario::default();
    s.committed(ada_in("Work/"));
    push_with_no_answer(&s);
    s.deleted(&oid(1));

    assert_eq!(push_accepting(&s), vec![]);
    assert_eq!(dropped(&s, DroppedBecause::Removed), 1);
    assert!(!attempted(&s));

    push_accepting(&s);
    assert_eq!(dropped(&s, DroppedBecause::Removed), 1, "left once");
}

#[test]
fn a_create_cut_off_before_any_answer_then_removed_is_noticed() {
    let s = Scenario::default();
    s.committed(ada_in("Work/"));
    let l = ScriptedLauncher::launching(|| ScriptedHelper {
        push_answer: Box::new(|_| Err(HelperError::Cancelled)),
        ..ScriptedHelper::answering(Ok(Default::default()))
    });
    s.dam().push(remotes_launched_by(&l), None).unwrap_err();
    s.deleted(&oid(1));
    push_accepting(&s);
    assert_eq!(dropped(&s, DroppedBecause::Removed), 1);
}

#[test]
fn a_removed_object_whose_create_was_refused_leaves_no_such_notice() {
    let s = Scenario::default();
    s.committed(ada_in("Work/"));
    let (l, _) = pushing(refusing_all("no"));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.deleted(&oid(1));
    push_accepting(&s);
    assert_eq!(dropped(&s, DroppedBecause::Removed), 0);
}

#[test]
fn an_unanswered_create_moved_out_of_the_remote_s_path_is_noticed_where_it_was_sent() {
    let s = scoped_to_work();
    s.committed(ada_in("Work/"));
    push_with_no_answer(&s);
    s.committed(ada_in("Home/"));

    assert_eq!(push_accepting(&s), vec![]);
    assert_eq!(dropped(&s, DroppedBecause::NoLongerSent), 1);
    assert!(!attempted(&s));
}

#[test]
fn a_held_object_moved_out_of_scope_leaves_no_such_notice_and_keeps_its_attempts() {
    let s = scoped_to_work();
    s.committed(ada_in("Work/"))
        .tracked_as(&oid(1), "r1")
        .pushed();
    s.committed(Object::Task(task_in(1, "Ada and tool", "Work/")));
    push_with_no_answer(&s);
    s.committed(Object::Task(task_in(1, "Ada and tool", "Home/")));

    assert_eq!(push_accepting(&s), vec![]);
    let notices = s.store.notices().unwrap();
    assert!(
        !notices
            .iter()
            .any(|n| matches!(n, Notice::UnansweredCreateDropped { .. })),
        "{notices:?}"
    );
    assert!(attempted(&s));
}

#[test]
fn an_unanswered_create_that_comes_back_after_its_notice_is_a_new_request() {
    let s = scoped_to_work();
    s.committed(ada_in("Work/"));
    let (l, first) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(ada_in("Home/"));
    push_accepting(&s);
    s.committed(ada_in("Work/"));

    let (l, again) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let (first, again) = (&first.borrow()[0], &again.borrow()[0]);
    assert_eq!(again.op, MutationOp::Create);
    assert_ne!(
        again.idempotency_key, first.idempotency_key,
        "the notice handed the first copy to the person"
    );
}

#[test]
fn an_unanswered_delete_whose_mapping_a_pull_cleared_is_forgotten_without_a_notice() {
    let s = Scenario::default();
    s.committed(ada_in("Work/"))
        .tracked_as(&oid(1), "r1")
        .pushed();
    s.deleted(&oid(1));
    push_with_no_answer(&s);
    let l = pulling(PullOutcome {
        removed: vec!["r1".into()],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();

    assert_eq!(push_accepting(&s), vec![]);
    let notices = s.store.notices().unwrap();
    assert!(
        !notices
            .iter()
            .any(|n| matches!(n, Notice::UnansweredCreateDropped { .. })),
        "{notices:?}"
    );
    assert!(!attempted(&s));
}
