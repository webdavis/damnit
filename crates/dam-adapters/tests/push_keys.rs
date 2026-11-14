use dam_application::testing::{
    Pushed, Scenario, ScriptedHelper, ScriptedLauncher, oid, remotes_launched_by, task, todoist,
};
use dam_application::{
    CommitRepository, Declared, HelperError, MutationOp, MutationOutcome, MutationVerdict,
    PushOutcome, RemoteMutation,
};
use dam_domain::{Object, Task};

type Answer = fn(&RemoteMutation) -> Option<MutationVerdict>;

fn the_wire_form(mutation: &RemoteMutation) -> Result<Declared, HelperError> {
    dam_adapters::declared_on_the_wire(mutation).map_err(|e| HelperError::Protocol(e.to_string()))
}

fn pushing(answer: Answer) -> (ScriptedLauncher, Pushed) {
    let pushed = Pushed::default();
    let seen = pushed.clone();
    let launcher = ScriptedLauncher::launching(move || ScriptedHelper {
        declare: the_wire_form,
        push_answer: Box::new(move |ms| {
            let outcomes = ms
                .iter()
                .filter_map(|m| {
                    let verdict = answer(m)?;
                    Some(MutationOutcome {
                        oid: m.oid.clone(),
                        verdict,
                    })
                })
                .collect();
            Ok(PushOutcome {
                outcomes,
                stopped_by: None,
            })
        }),
        pushed: seen.clone(),
        ..ScriptedHelper::answering(Ok(Default::default()))
    });
    (launcher, pushed)
}

fn no_answer(_: &RemoteMutation) -> Option<MutationVerdict> {
    None
}

fn accepted(_: &RemoteMutation) -> Option<MutationVerdict> {
    Some(MutationVerdict::Accepted {
        remote_id: Some("r1".into()),
    })
}

fn refused(_: &RemoteMutation) -> Option<MutationVerdict> {
    Some(MutationVerdict::Refused {
        why: "not now".into(),
        unasked: false,
    })
}

fn named(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

fn held_as_t1() -> Scenario {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s
}

fn key_of_one_push(s: &Scenario, answer: Answer) -> (MutationOp, String) {
    let (l, pushed) = pushing(answer);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 1, "{sent:?}");
    (sent[0].op, sent[0].idempotency_key.clone())
}

#[test]
fn an_update_s_key_is_pinned_for_its_wire_form() {
    let s = held_as_t1();
    s.committed(named("second"));
    assert_eq!(
        key_of_one_push(&s, accepted),
        (
            MutationOp::Update,
            "4e46b1da-a777-8228-8927-faed04312200".to_string()
        )
    );
}

#[test]
fn a_create_resent_after_no_answer_carries_its_first_key_through_an_edit() {
    let s = Scenario::default();
    s.committed(task(1));
    let first = key_of_one_push(&s, no_answer);
    assert_eq!(key_of_one_push(&s, no_answer), first);
    s.committed(named("edited"));
    assert_eq!(key_of_one_push(&s, no_answer), first);
    assert_eq!(first.0, MutationOp::Create);
}

#[test]
fn the_same_update_resent_after_no_answer_carries_the_same_key() {
    let s = held_as_t1();
    s.committed(named("second"));
    let first = key_of_one_push(&s, no_answer);
    assert_eq!(key_of_one_push(&s, no_answer), first);
    assert_eq!(first.0, MutationOp::Update);
}

#[test]
fn an_update_set_back_after_two_unanswered_ones_gets_a_key_of_its_own() {
    let s = held_as_t1();
    s.committed(named("x"));
    let first = key_of_one_push(&s, no_answer);
    s.committed(named("y"));
    let second = key_of_one_push(&s, no_answer);
    s.committed(named("x"));
    let third = key_of_one_push(&s, no_answer);
    assert_ne!(
        third, first,
        "the remote would skip the set-back as already run"
    );
    assert_ne!(third, second);
}

#[test]
fn two_accepted_changes_to_one_object_carry_different_keys() {
    let s = held_as_t1();
    s.committed(named("first"));
    let first = key_of_one_push(&s, accepted);
    s.committed(named("second"));
    assert_ne!(key_of_one_push(&s, accepted), first);
}

#[test]
fn two_retries_of_one_object_for_different_changes_carry_different_keys() {
    let s = held_as_t1();
    s.committed(named("first")).pushed();
    s.store.set_push_retries(&todoist(), &[oid(1)]).unwrap();
    let first = key_of_one_push(&s, refused);
    s.committed(named("second")).pushed();
    assert_ne!(key_of_one_push(&s, refused), first);
}

#[test]
fn a_refused_create_retried_carries_a_new_key_each_time() {
    let s = Scenario::default();
    s.committed(task(1));
    let first = key_of_one_push(&s, refused);
    let second = key_of_one_push(&s, refused);
    assert_ne!(second, first);
    assert_eq!(second.0, MutationOp::Create);
}

#[test]
fn a_delete_sent_again_after_something_else_was_attempted_gets_a_key_of_its_own() {
    let s = held_as_t1();
    s.deleted(&oid(1));
    let first = key_of_one_push(&s, no_answer);
    assert_eq!(key_of_one_push(&s, no_answer), first);
    s.committed(named("restored"));
    assert_eq!(key_of_one_push(&s, no_answer).0, MutationOp::Update);
    s.deleted(&oid(1));
    let again = key_of_one_push(&s, no_answer);
    assert_eq!(again.0, MutationOp::Delete);
    assert_ne!(again, first);
}
