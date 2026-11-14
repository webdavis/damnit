mod support;

use dam_application::testing::prelude::*;
use dam_application::testing::{Scenario, oid, remotes_launched_by, task_in, todoist};
use dam_domain::Object;
use dam_helper::testing;
use support::real_helper::the_real_helper_against;
use support::stateful_todoist::{Fault, serve, work_held_upstream};

#[test]
fn a_create_in_doubt_whose_resend_is_unauthorized_lands_once() {
    let _guard = testing::guard("a_create_in_doubt_whose_resend_is_unauthorized_lands_once")
        .allow_slow("launches the real helper process three times");
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "Ada", "Work/")));
    let held = work_held_upstream()
        .applying_the_next_write_then(Fault::HangUp)
        .unauthorized_after(1);
    let (server, upstream) = serve(held);
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());
    s.dam().push(remotes_launched_by(&launcher), None).unwrap();

    let refused = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);
    assert!(refused.failed[0].1.contains("401"), "{:?}", refused.failed);

    let report = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);

    assert!(report.failed.is_empty(), "{:?}", report.failed);
    let upstream = upstream.lock().unwrap();
    assert_eq!(upstream.item("Ada").len(), 1, "{:?}", upstream.items);
    assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_some());
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
}
