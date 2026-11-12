use dam_domain::Permission::{TaskChange, TaskCreate};
use dam_domain::{Author, Field, ItemHash, Object, Outcome, Path, Priority, Task};

use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{FixedRandom, Scenario, allowed, holder, oid, task_in};
use crate::use_cases::edit::EditFields;
use crate::use_cases::items::land::Landed;
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::new_object::NewTask;
use crate::use_cases::prefix::Named;

fn sent_hash(landed: Landed) -> ItemHash {
    match landed {
        Landed::Sent(sent) => sent.item.hash,
        Landed::Written(object) => panic!("written: {object:?}"),
    }
}

fn dentist(s: &Scenario) -> ItemHash {
    let vpt = holder(vec![allowed(&[TaskCreate], None)]);
    let dam = Dam {
        access: &vpt,
        ..s.dam()
    };
    sent_hash(
        dam.new_task(NewTask {
            subject: "call the dentist".into(),
            path: Path::parse("inbox").unwrap(),
            priority: Priority::default(),
            due: None,
            deadline: None,
            labels: Default::default(),
            body: String::new(),
        })
        .unwrap(),
    )
}

fn retitled_by_vpt(s: &Scenario, subject: &str) -> ItemHash {
    let vpt = holder(vec![allowed(&[TaskChange], None)]);
    let dam = Dam {
        access: &vpt,
        ..s.dam()
    };
    let fields = EditFields {
        subject: Some(subject.into()),
        ..EditFields::default()
    };
    sent_hash(dam.edit(&oid(1), fields).unwrap())
}

fn rent_with(subject: &str, body: &str) -> Object {
    let mut task: Task = task_in(1, subject, "home");
    task.base.body = body.into();
    Object::Task(task)
}

#[test]
fn adding_a_new_item_creates_its_object_under_the_items_hash_and_stages_it_as_the_tokens() {
    let s = Scenario::default().drawing_from(7);
    let hash = dentist(&s);
    let staged = s.dam().add(&[Named::Item(hash.clone())]).unwrap();
    let created = s.store.get(hash.oid()).unwrap().unwrap();
    assert_eq!(created.base().subject, "call the dentist");
    assert_eq!(staged[0].change.oid(), hash.oid());
    assert_eq!(staged[0].author.token().unwrap().name, "vpt");
    assert!(matches!(
        s.store.item(&hash).unwrap().unwrap().outcome,
        Outcome::Added(_)
    ));
    let record = s.dam().commit("from vpt").unwrap();
    assert_eq!(record.changes, staged);
}

#[test]
fn a_change_item_takes_only_its_fields_and_leaves_the_owners_other_edit_unstaged() {
    let s = Scenario::default().drawing_from(7);
    s.committed(rent_with("pay rent", ""));
    let hash = retitled_by_vpt(&s, "pay the rent");
    s.working(rent_with("pay rent", "by friday"));
    let staged = s.dam().add(&[Named::Item(hash)]).unwrap();
    let after = staged[0].change.after().unwrap().base();
    assert_eq!(
        (after.subject.as_str(), after.body.as_str()),
        ("pay the rent", "")
    );
    assert!(matches!(staged[0].author, Author::Token(_)));
    let working = s.store.get(&oid(1)).unwrap().unwrap();
    assert_eq!(working.base().subject, "pay the rent");
    assert_eq!(working.base().body, "by friday");
}

#[test]
fn a_change_to_a_field_the_owner_edited_and_has_not_saved_stops_the_add() {
    let s = Scenario::default().drawing_from(7);
    s.committed(rent_with("pay rent", ""));
    let hash = retitled_by_vpt(&s, "pay the rent");
    s.working(rent_with("pay rent now", ""));
    assert_eq!(
        s.dam().add(&[Named::Item(hash.clone())]),
        Err(ItemRefusal::Clash {
            hash: hash.clone(),
            target: oid(1),
            fields: vec![Field::Subject],
        }
        .into())
    );
    assert!(s.store.item(&hash).unwrap().unwrap().is_waiting());
    assert!(s.store.staged().unwrap().is_empty());
}

#[test]
fn add_all_takes_every_item_that_applies_and_leaves_a_clashing_one_waiting() {
    let mut s = Scenario::default().drawing_from(7);
    s.committed(rent_with("pay rent", ""));
    let clashing = retitled_by_vpt(&s, "pay the rent");
    s.working(rent_with("pay rent now", ""));
    s.fixture.random = FixedRandom::new(8);
    let new = dentist(&s);
    let staged = s.dam().add_all().unwrap();
    let oids: Vec<_> = staged.iter().map(|a| a.change.oid().clone()).collect();
    assert_eq!(oids, vec![new.oid().clone(), oid(1)]);
    assert_eq!(staged[1].author, Author::Owner);
    assert!(s.store.item(&clashing).unwrap().unwrap().is_waiting());
}

#[test]
fn an_item_already_settled_is_not_added_again() {
    let s = Scenario::default().drawing_from(7);
    let hash = dentist(&s);
    s.dam().add(&[Named::Item(hash.clone())]).unwrap();
    assert_eq!(
        s.dam().add(&[Named::Item(hash.clone())]),
        Err(ItemRefusal::NotWaiting {
            hash,
            outcome: "added"
        }
        .into())
    );
}

#[test]
fn a_prefix_names_a_waiting_item_and_an_object_verb_refuses_it() {
    let s = Scenario::default().drawing_from(7);
    let hash = dentist(&s);
    let prefix = &hash.as_str()[..6];
    assert_eq!(s.dam().resolve_named(prefix), Ok(Named::Item(hash.clone())));
    assert_eq!(
        s.dam().resolve_object(prefix),
        Err(ItemRefusal::NotAnObject(hash).into())
    );
}

mod blocked_after_merge;
mod removal_clash;
