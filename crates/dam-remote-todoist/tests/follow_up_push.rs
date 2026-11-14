mod support;

use dam_application::ObjectRepository;
use dam_application::testing::{
    FaultyStore, FixedRandom, Fixture, MemoryStore, Scenario, config, remotes_launched_by, task_in,
};
use dam_domain::Object;
use dam_helper::testing;
use support::real_helper::the_real_helper_against;
use support::stateful_todoist::{Fault, serve, work_held_upstream};

fn ada(subject: &str) -> Object {
    Object::Task(task_in(1, subject, "Work/"))
}

fn commit(store: &MemoryStore, object: Object) {
    let arranging = Fixture {
        random: FixedRandom::new(200),
        ..Fixture::default()
    };
    store.put(&object).unwrap();
    arranging.dam(store).add_all().unwrap();
    arranging.dam(store).commit("m").unwrap();
}

#[test]
fn a_create_edited_before_its_resend_lands_once_holding_the_edit() {
    let _guard = testing::guard("a_create_edited_before_its_resend_lands_once_holding_the_edit")
        .allow_slow("launches the real helper process twice");
    let s = Scenario::default();
    s.committed(ada("Ada"));
    let (server, upstream) =
        serve(work_held_upstream().applying_the_next_write_then(Fault::HangUp));
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());
    s.dam().push(remotes_launched_by(&launcher), None).unwrap();
    upstream.lock().unwrap().items[0]["priority"] = 4.into();

    s.committed(ada("Ada and tool"));
    let report = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);

    assert!(report.failed.is_empty(), "{:?}", report.failed);
    let upstream = upstream.lock().unwrap();
    assert_eq!(upstream.items.len(), 1, "{:?}", upstream.items);
    assert_eq!(
        upstream.item("Ada and tool").len(),
        1,
        "{:?}",
        upstream.items
    );
    assert_eq!(
        upstream.items[0]["priority"], 4,
        "the follow-up wrote a field dam never changed: {:?}",
        upstream.items
    );
}

#[test]
fn a_create_applied_whose_settlement_failed_lands_once_with_the_edit_made_after() {
    let _guard = testing::guard(
        "a_create_applied_whose_settlement_failed_lands_once_with_the_edit_made_after",
    )
    .allow_slow("launches the real helper process twice");
    let store = FaultyStore::new(MemoryStore::new());
    let fixture = Fixture {
        config: config(),
        ..Fixture::default()
    };
    commit(&store.inner, ada("Ada"));
    let (server, upstream) = serve(work_held_upstream());
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());
    store.fail_at_write(1);
    let failed = fixture
        .dam(&store)
        .push(remotes_launched_by(&launcher), None);
    assert!(failed.is_err(), "the settlement write failed: {failed:?}");
    store.fail_at_write(usize::MAX);

    commit(&store.inner, ada("Ada and tool"));
    fixture
        .dam(&store)
        .push(remotes_launched_by(&launcher), None)
        .unwrap();

    let upstream = upstream.lock().unwrap();
    assert_eq!(upstream.items.len(), 1, "{:?}", upstream.items);
    assert_eq!(
        upstream.item("Ada and tool").len(),
        1,
        "{:?}",
        upstream.items
    );
}
