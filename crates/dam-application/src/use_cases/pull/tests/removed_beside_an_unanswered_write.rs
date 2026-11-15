use dam_domain::{Object, Task};

use super::unanswered::{pull_outcome, push_accepting, push_with_no_answer};
use crate::remote::{MutationOp, PullOutcome};
use crate::testing::{Scenario, oid};

fn ann(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

fn bob(subject: &str) -> Object {
    Object::Task(Task::new(oid(2), subject))
}

fn ann_removed_in_the_app_while_bob_waits() -> Scenario {
    let s = Scenario::default();
    s.committed(ann("Ann")).tracked_as(&oid(1), "i:1");
    s.committed(bob("Bob")).tracked_as(&oid(2), "i:2").pushed();
    s.working(ann("Ann 2"));
    s.committed(bob("Bob 2"));
    push_with_no_answer(&s);
    let removed = PullOutcome {
        removed: vec!["i:1".into()],
        ..PullOutcome::default()
    };
    pull_outcome(&s, removed);
    push_with_no_answer(&s);
    s
}

fn bob_renamed() -> (MutationOp, Option<String>, Option<String>) {
    (MutationOp::Update, Some("i:2".into()), Some("Bob 2".into()))
}

#[test]
fn a_task_removed_in_the_app_stays_removed_after_a_push_that_leaves_its_commit_unpushed() {
    let s = ann_removed_in_the_app_while_bob_waits();

    assert_eq!(push_accepting(&s, "i:9"), vec![bob_renamed()]);
}

#[test]
fn an_edit_after_that_sends_the_removed_task_as_one_create() {
    let s = ann_removed_in_the_app_while_bob_waits();

    s.committed(ann("Ann 3"));

    let create = (MutationOp::Create, None, Some("Ann 3".into()));
    let sent = push_accepting(&s, "i:9");
    assert_eq!(sent.len(), 2, "{sent:?}");
    assert!(
        sent.contains(&create) && sent.contains(&bob_renamed()),
        "{sent:?}"
    );
}
