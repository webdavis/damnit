use super::clash::clashing_fields;
use crate::{Field, Object, Oid, Task};

fn rent(subject: &str, body: &str) -> Object {
    let mut task = Task::new(Oid::generate(&mut |b| b.fill(1)), subject);
    task.base.body = body.into();
    Object::Task(task)
}

#[test]
fn a_field_the_owner_left_alone_takes_the_items_value_without_a_clash() {
    let committed = rent("pay rent", "");
    let after = rent("pay the rent", "");
    let working = rent("pay rent", "by friday");
    assert!(clashing_fields(&committed, &after, &committed, &working).is_empty());
}

#[test]
fn a_field_the_owner_changed_and_has_not_saved_clashes() {
    let committed = rent("pay rent", "");
    let after = rent("pay the rent", "");
    let working = rent("pay rent now", "");
    assert_eq!(
        clashing_fields(&committed, &after, &committed, &working),
        vec![Field::Subject]
    );
}

#[test]
fn a_field_committed_since_the_tool_looked_clashes() {
    let seen = rent("pay rent", "");
    let after = rent("pay the rent", "");
    let committed = rent("pay rent now", "");
    assert_eq!(
        clashing_fields(&seen, &after, &committed, &committed),
        vec![Field::Subject]
    );
}

#[test]
fn an_owner_edit_that_already_agrees_with_the_item_is_no_clash() {
    let committed = rent("pay rent", "");
    let after = rent("pay the rent", "");
    assert!(clashing_fields(&committed, &after, &committed, &after).is_empty());
}
