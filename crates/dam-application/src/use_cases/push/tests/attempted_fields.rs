use dam_domain::{Completion, Field, Object, Path, Task};

use crate::ports::{AttemptRepository, HelperError};
use crate::remote::MutationOp;
use crate::testing::{
    Scenario, ScriptedHelper, ScriptedLauncher, accepting_all, config, oid, pushing,
    remotes_launched_by, task, task_in, todoist,
};

fn held_upstream_as_t1() -> Scenario {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s
}

fn renamed(subject: &str) -> Task {
    Task::new(oid(1), subject)
}

fn push_with_no_answer(s: &Scenario) {
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
}

fn push_cut_off_before_any_answer(s: &Scenario) {
    let l = ScriptedLauncher::launching(|| ScriptedHelper {
        push_answer: Box::new(|_| Err(HelperError::Cancelled)),
        ..ScriptedHelper::answering(Ok(Default::default()))
    });
    s.dam().push(remotes_launched_by(&l), None).unwrap_err();
}

fn the_one_write_sent_next(s: &Scenario) -> (MutationOp, Vec<Field>, Option<Object>) {
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 1, "{sent:?}");
    (sent[0].op, sent[0].fields.clone(), sent[0].object.clone())
}

#[test]
fn a_field_set_back_after_its_unanswered_update_is_sent() {
    let s = held_upstream_as_t1();
    s.committed(Object::Task(renamed("x")));
    push_with_no_answer(&s);
    s.committed(task(1));
    assert_eq!(
        the_one_write_sent_next(&s),
        (MutationOp::Update, vec![Field::Subject], Some(task(1)))
    );
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
}

#[test]
fn a_field_set_back_after_a_push_cut_off_before_any_answer_is_sent() {
    let s = held_upstream_as_t1();
    s.committed(Object::Task(renamed("x")));
    push_cut_off_before_any_answer(&s);
    s.committed(task(1));
    assert_eq!(
        the_one_write_sent_next(&s),
        (MutationOp::Update, vec![Field::Subject], Some(task(1)))
    );
}

#[test]
fn the_next_push_declares_what_an_attempt_may_have_changed_and_its_own_difference_only() {
    let s = held_upstream_as_t1();
    s.committed(Object::Task(renamed("x")));
    push_with_no_answer(&s);
    let mut done = renamed("t1");
    done.completion = Completion::Done { at: None };
    s.committed(Object::Task(done));
    let (_, fields, _) = the_one_write_sent_next(&s);
    assert_eq!(fields, vec![Field::Subject, Field::Done]);
}

#[test]
fn a_held_object_moved_out_of_scope_and_back_declares_what_its_unanswered_update_sent() {
    let mut config = config();
    config.remotes[0].path = Some(Path::parse("Work").unwrap());
    let s = Scenario::default().configured(config);
    let ada = |subject: &str, path: &str| Object::Task(task_in(1, subject, path));
    s.committed(ada("Ada", "Work/"))
        .tracked_as(&oid(1), "r1")
        .pushed();
    s.committed(ada("Ada and tool", "Work/"));
    push_with_no_answer(&s);
    s.committed(ada("Ada and tool", "Home/"));
    let (l, _) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();

    s.committed(ada("Ada and tool", "Work/"));
    let (_, fields, _) = the_one_write_sent_next(&s);
    assert_eq!(fields, vec![Field::Subject]);
}
