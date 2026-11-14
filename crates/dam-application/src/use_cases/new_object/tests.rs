use super::*;
use crate::config::Config;
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, FixedRandom, Fixture, MemoryStore, Scenario, oid};
use dam_domain::{Categories, Category};

fn input(subject: &str, labels: &[&str]) -> NewTask {
    NewTask {
        subject: subject.into(),
        path: Path::root(),
        priority: Priority::default(),
        due: None,
        deadline: None,
        labels: labels.iter().map(|s| s.to_string()).collect(),
        body: String::new(),
        id: None,
    }
}

fn effort() -> Categories {
    Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["light".into(), "deep".into()],
        exclusive: true,
    }])
    .unwrap()
}

fn dentist(start: When, end: When) -> NewEvent {
    NewEvent {
        subject: "dentist".into(),
        path: Path::root(),
        start,
        end,
        labels: Default::default(),
        body: String::new(),
        id: None,
    }
}

fn written_oid(landed: Landed) -> Oid {
    match landed {
        Landed::Written(object) => object.oid().clone(),
        Landed::Sent(sent) => panic!("the owner's object was sent as {sent:?}"),
    }
}

fn drawing_from(byte: u8) -> Fixture {
    Fixture {
        random: FixedRandom::new(byte),
        ..Fixture::default()
    }
}

#[test]
fn a_new_task_lands_in_working_with_a_generated_oid_and_nothing_staged() {
    let s = Scenario::default().drawing_from(7);
    let id = written_oid(s.dam().new_task(input("milk", &[])).unwrap());
    assert_eq!(id, oid(7));
    let got = s.store.get(&id).unwrap().unwrap();
    assert_eq!(got.base().subject, "milk");
    assert!(s.store.staged().unwrap().is_empty());
    assert!(s.store.committed(&id).unwrap().is_none());
}

#[test]
fn labels_are_checked_against_categories() {
    let s = Scenario::default().drawing_from(1).configured(Config {
        categories: effort(),
        ..Config::default()
    });
    let err = s
        .dam()
        .new_task(input("x", &["light", "deep"]))
        .unwrap_err();
    assert!(matches!(err, UseCaseError::Refused(Refusal::Labels(_))));
    assert!(s.store.all().unwrap().is_empty());
}

#[test]
fn a_new_event_lands_in_working() {
    let s = Scenario::default().drawing_from(2);
    let start = When::Day(jiff::civil::date(2026, 9, 25));
    let end = When::Day(jiff::civil::date(2026, 9, 26));
    let id = written_oid(s.dam().new_event(dentist(start, end)).unwrap());
    assert!(matches!(s.store.get(&id).unwrap(), Some(Object::Event(_))));
}

#[test]
fn a_new_task_that_fails_at_its_write_leaves_nothing_behind() {
    let store = FaultyStore::new(MemoryStore::new());
    let fixture = drawing_from(7);
    let dam = fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.new_task(input("milk", &[])).map(drop));
}

#[test]
fn a_new_event_that_fails_at_its_write_leaves_nothing_behind() {
    let store = FaultyStore::new(MemoryStore::new());
    let day = When::Day(jiff::civil::date(2026, 9, 25));
    let fixture = drawing_from(2);
    let dam = fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| {
        dam.new_event(dentist(day.clone(), day.clone())).map(drop)
    });
}
