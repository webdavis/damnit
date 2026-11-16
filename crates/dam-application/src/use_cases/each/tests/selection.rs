use super::*;
use crate::errors::{Refusal, UseCaseError};

#[test]
fn named_oids_are_the_set() {
    let s = work_and_home();
    let out = s
        .dam()
        .edit_each(&named(&[1, 3]), retitled("x"), run())
        .unwrap();
    assert_eq!(out.oids, oids(&[1, 3]));
    assert_eq!(changed(&out), oids(&[1, 3]));
    assert_eq!(subject_of(&s, 2), "invoice");
}

#[test]
fn an_oid_named_twice_is_changed_once() {
    let s = work_and_home();
    let out = s
        .dam()
        .edit_each(&named(&[1, 1]), retitled("x"), run())
        .unwrap();
    assert_eq!(out.oids, oids(&[1]));
}

#[test]
fn a_query_selects_what_ls_lists() {
    let s = work_and_home();
    let out = s
        .dam()
        .edit_each(&query("path:work/"), retitled("x"), run())
        .unwrap();
    let mut selected = out.oids.clone();
    selected.sort();
    assert_eq!(selected, oids(&[1, 2]));
}

#[test]
fn a_query_never_reads_a_word_as_a_filter_name() {
    let s = work_and_home();
    assert!(
        s.dam()
            .edit_each(&query("home"), retitled("x"), run())
            .is_err()
    );
    assert_eq!(subject_of(&s, 3), "laundry");
}

#[test]
fn a_filter_selects_by_its_saved_query() {
    let s = work_and_home();
    let filter = Selector::Filter("home".into());
    let out = s.dam().edit_each(&filter, retitled("x"), run()).unwrap();
    assert_eq!(out.oids, oids(&[3]));
}

#[test]
fn an_unknown_filter_is_refused() {
    let s = work_and_home();
    let filter = Selector::Filter("nope".into());
    assert_eq!(
        s.dam().edit_each(&filter, retitled("x"), run()),
        Err(UseCaseError::Refused(Refusal::NoSuchFilter("nope".into())))
    );
}

#[test]
fn an_empty_set_changes_nothing_and_counts_zero() {
    let s = work_and_home();
    let out = s
        .dam()
        .edit_each(&query("path:garden/"), retitled("x"), run())
        .unwrap();
    assert!(out.oids.is_empty() && out.changes.is_empty());
}

#[test]
fn an_object_already_as_asked_has_no_change() {
    let s = work_and_home();
    let out = s
        .dam()
        .edit_each(&named(&[1, 2]), retitled("report"), run())
        .unwrap();
    assert_eq!(changed(&out), oids(&[2]));
}

#[test]
fn one_object_that_breaks_a_rule_changes_nothing() {
    let s = work_and_home();
    let before = s.store.clone();
    let fields = EditFields {
        add_depends: vec![oid(1)],
        ..EditFields::default()
    };
    assert!(s.dam().edit_each(&named(&[2, 1]), fields, run()).is_err());
    assert_eq!(s.store, before);
}

#[test]
fn an_expect_that_differs_from_the_count_is_refused_before_anything_changes() {
    let s = work_and_home();
    let before = s.store.clone();
    let expecting = SetRun {
        expect: Some(3),
        ..run()
    };
    assert_eq!(
        s.dam()
            .edit_each(&query("path:work/"), retitled("x"), expecting),
        Err(UseCaseError::Refused(Refusal::CountMismatch {
            expected: 3,
            count: 2
        }))
    );
    assert_eq!(s.store, before);
}

#[test]
fn a_matching_expect_runs() {
    let s = work_and_home();
    let expecting = SetRun {
        expect: Some(2),
        ..run()
    };
    assert!(
        s.dam()
            .edit_each(&query("path:work/"), retitled("x"), expecting)
            .is_ok()
    );
}
