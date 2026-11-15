use super::unanswered::{
    named, pull, pulled, push_accepting, push_answering_creates_only, push_with_no_answer, subject,
};
use crate::ports::AttemptRepository;
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, todoist};

fn renamed_with_no_answer(s: &Scenario) {
    s.committed(named("a")).tracked_as(&oid(1), "i:1").pushed();
    s.committed(named("b"));
    push_with_no_answer(s);
}

fn update(to: &str) -> (MutationOp, Option<String>, Option<String>) {
    (MutationOp::Update, Some("i:1".into()), Some(to.into()))
}

#[test]
fn a_revert_here_survives_a_pull_of_dams_own_unanswered_rename() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);
    s.committed(named("a"));

    pull(&s, vec![pulled(named("b"), "i:1")]);

    assert_eq!(subject(&s), "a");
    assert_eq!(push_accepting(&s, "i:1"), vec![update("a")]);
}

#[test]
fn dams_own_unanswered_rename_pulled_back_is_no_conflict() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);
    s.committed(named("c"));

    pull(&s, vec![pulled(named("b"), "i:1")]);

    assert!(s.store.conflicts().unwrap().is_empty());
    assert_eq!(subject(&s), "c");
    assert_eq!(push_accepting(&s, "i:1"), vec![update("c")]);
}

#[test]
fn a_value_dam_never_sent_is_taken_from_the_app() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);
    s.committed(named("a"));

    pull(&s, vec![pulled(named("z"), "i:1")]);

    assert_eq!(subject(&s), "z");
}

#[test]
fn a_pull_after_an_unanswered_follow_up_keeps_the_rename() {
    let s = Scenario::default();
    s.committed(named("a"));
    push_with_no_answer(&s);
    s.committed(named("b"));
    push_answering_creates_only(&s, "i:1");

    pull(&s, vec![pulled(named("a"), "i:1")]);

    assert_eq!(subject(&s), "b");
    assert_eq!(push_accepting(&s, "i:1"), vec![update("b")]);
}

#[test]
fn a_pull_that_finds_dams_write_landed_settles_it() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);

    pull(&s, vec![pulled(named("b"), "i:1")]);

    assert_eq!(push_accepting(&s, "i:1"), vec![]);
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
}
