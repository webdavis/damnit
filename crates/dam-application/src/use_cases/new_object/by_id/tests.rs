use dam_domain::{CallerId, Path, Priority};

use crate::ports::ObjectRepository;
use crate::testing::{FaultyStore, FixedRandom, Fixture, MemoryStore};
use crate::use_cases::new_object::NewTask;

fn dentist(subject: &str) -> NewTask {
    NewTask {
        subject: subject.into(),
        path: Path::root(),
        priority: Priority::default(),
        due: None,
        deadline: None,
        labels: Default::default(),
        body: String::new(),
        id: Some(CallerId::parse("dentist").unwrap()),
    }
}

fn fixture() -> Fixture {
    Fixture {
        random: FixedRandom::new(7),
        ..Fixture::default()
    }
}

#[test]
fn a_first_new_by_id_that_fails_at_any_write_leaves_nothing_behind() {
    let store = FaultyStore::new(MemoryStore::new());
    let fixture = fixture();
    let dam = fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.new_task(dentist("Call dentist")).map(drop));
}

#[test]
fn a_repeat_by_id_that_fails_at_any_write_leaves_the_task_as_it_was() {
    let store = FaultyStore::new(MemoryStore::new());
    let fixture = fixture();
    let dam = fixture.dam(&store);
    dam.new_task(dentist("Call dentist")).unwrap();
    store.assert_unchanged_when_any_write_fails(|| {
        dam.new_task(dentist("Call the dentist")).map(drop)
    });
    let all = store.all().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].base().subject, "Call the dentist");
}
