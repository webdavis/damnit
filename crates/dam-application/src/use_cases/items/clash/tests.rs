use dam_domain::Permission::TaskChange;
use dam_domain::{Author, ItemHash, Object, Outcome};

use crate::errors::Refusal;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid, task_in};
use crate::use_cases::edit::EditFields;
use crate::use_cases::items::land::Landed;
use crate::use_cases::prefix::Named;
use crate::use_cases::resolve::Side;

fn rent(subject: &str) -> Object {
    Object::Task(task_in(1, subject, "home"))
}

fn clashing_retitle() -> (Scenario, ItemHash) {
    let s = Scenario::default().drawing_from(7);
    s.committed(rent("pay rent"));
    let vpt = holder(vec![allowed(&[TaskChange], None)]);
    let dam = Dam {
        access: &vpt,
        ..s.dam()
    };
    let fields = EditFields {
        subject: Some("pay the rent".into()),
        ..EditFields::default()
    };
    let Landed::Sent(sent) = dam.edit(&oid(1), fields).unwrap() else {
        panic!("written")
    };
    s.working(rent("pay rent now"));
    (s, sent.item.hash)
}

#[test]
fn theirs_takes_the_items_values_over_the_owners_unsaved_edit_and_stages_them() {
    let (s, hash) = clashing_retitle();
    assert_eq!(
        s.dam().settle(&Named::Item(hash.clone()), Side::Theirs),
        Ok(oid(1))
    );
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "pay the rent"
    );
    let staged = s.store.staged().unwrap();
    assert!(matches!(staged[0].author, Author::Token(_)));
    assert!(matches!(
        s.store.item(&hash).unwrap().unwrap().outcome,
        Outcome::Added(_)
    ));
}

#[test]
fn ours_named_by_the_task_declines_the_item_and_keeps_the_owners_edit() {
    let (s, hash) = clashing_retitle();
    assert_eq!(
        s.dam().settle(&Named::Object(oid(1)), Side::Ours),
        Ok(oid(1))
    );
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "pay rent now"
    );
    assert!(s.store.staged().unwrap().is_empty());
    assert!(matches!(
        s.store.item(&hash).unwrap().unwrap().outcome,
        Outcome::Declined(_)
    ));
}

#[test]
fn a_task_with_no_conflict_and_no_clashing_item_has_nothing_to_resolve() {
    let s = Scenario::default();
    s.committed(rent("pay rent"));
    assert_eq!(
        s.dam().settle(&Named::Object(oid(1)), Side::Ours),
        Err(Refusal::NoConflict(oid(1)).into())
    );
}
