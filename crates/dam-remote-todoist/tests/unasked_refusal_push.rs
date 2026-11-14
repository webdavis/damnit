mod support;

use dam_application::testing::{Scenario, remotes_launched_by, task_in};
use dam_domain::Object;
use dam_helper::testing;
use support::real_helper::the_real_helper_against;
use support::stateful_todoist::{Fault, serve, work_held_upstream};

#[test]
fn a_create_in_doubt_refused_before_todoist_was_asked_lands_once() {
    let _guard = testing::guard("a_create_in_doubt_refused_before_todoist_was_asked_lands_once")
        .allow_slow("launches the real helper process three times");
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "Ada", "Work/")));
    let (server, upstream) =
        serve(work_held_upstream().applying_the_next_write_then(Fault::HangUp));
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());
    s.dam().push(remotes_launched_by(&launcher), None).unwrap();

    upstream.lock().unwrap().projects[0]["name"] = "Elsewhere".into();
    let refused = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);
    assert!(
        refused.failed[0].1.contains("Work"),
        "the helper found no project to place it in: {:?}",
        refused.failed
    );

    upstream.lock().unwrap().projects[0]["name"] = "Work".into();
    s.dam().push(remotes_launched_by(&launcher), None).unwrap();

    let upstream = upstream.lock().unwrap();
    assert_eq!(upstream.item("Ada").len(), 1, "{:?}", upstream.items);
}
