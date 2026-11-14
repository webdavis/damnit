mod support;

use dam_application::testing::{Scenario, remotes_launched_by, task_in};
use dam_domain::Object;
use dam_helper::testing;
use support::real_helper::the_real_helper_against;
use support::stateful_todoist::{Fault, serve, work_held_upstream};

fn ada(subject: &str) -> Object {
    Object::Task(task_in(1, subject, "Work/"))
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
}

#[test]
fn a_field_reverted_after_its_unanswered_update_is_set_back_upstream() {
    let _guard =
        testing::guard("a_field_reverted_after_its_unanswered_update_is_set_back_upstream")
            .allow_slow("launches the real helper process three times");
    let s = Scenario::default();
    s.committed(ada("Ada"));
    let (server, upstream) = serve(work_held_upstream());
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());
    s.dam().push(remotes_launched_by(&launcher), None).unwrap();

    upstream
        .lock()
        .unwrap()
        .fail_after_the_next_write(Fault::HangUp);
    s.committed(ada("Ada and tool"));
    s.dam().push(remotes_launched_by(&launcher), None).unwrap();
    assert_eq!(upstream.lock().unwrap().item("Ada and tool").len(), 1);

    s.committed(ada("Ada"));
    let report = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);

    assert!(report.failed.is_empty(), "{:?}", report.failed);
    let upstream = upstream.lock().unwrap();
    assert_eq!(upstream.items.len(), 1, "{:?}", upstream.items);
    assert_eq!(upstream.item("Ada").len(), 1, "{:?}", upstream.items);
}
