mod support;

use dam_application::testing::prelude::*;
use dam_application::testing::{Scenario, oid, remotes_launched_by, task_in, todoist};
use dam_domain::{Completion, Object};
use dam_helper::testing;
use serde_json::json;
use support::real_helper::{FRESH_HELPER_PROCESSES_TRIED, the_real_helper_against};
use support::stateful_todoist::{Fault, Held, serve};

fn work_held_upstream() -> Held {
    Held::new(
        vec![
            json!({"id": "work", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}),
        ],
        vec![],
        vec![],
    )
}

fn a_create_whose_write_got_no_answer_lands_once_on_the_next_push(fault: Fault) {
    for _ in 0..FRESH_HELPER_PROCESSES_TRIED {
        one_round(fault);
    }
}

fn one_round(fault: Fault) {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "Ada", "Work/")));
    let (server, upstream) = serve(work_held_upstream().applying_the_next_write_then(fault));
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());

    s.dam().push(remotes_launched_by(&launcher), None).unwrap();
    assert_eq!(
        s.store.unpushed(&todoist()).unwrap().len(),
        1,
        "a commit whose write got no answer is not marked pushed"
    );
    let report = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);

    assert!(report.failed.is_empty(), "{:?}", report.failed);
    let upstream = upstream.lock().unwrap();
    assert_eq!(upstream.item("Ada").len(), 1, "{:?}", upstream.items);
    let received = &upstream.uuids_received;
    assert_eq!(received.len(), 2, "{received:?}");
    assert_eq!(received[0], received[1], "the resend minted a fresh uuid");
    assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_some());
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
}

#[test]
fn a_create_whose_connection_dropped_after_it_was_applied_lands_once() {
    let _guard =
        testing::guard("a_create_whose_connection_dropped_after_it_was_applied_lands_once")
            .allow_slow("each round launches the real helper process twice");
    a_create_whose_write_got_no_answer_lands_once_on_the_next_push(Fault::HangUp);
}

#[test]
fn a_create_answered_by_a_server_error_after_it_was_applied_lands_once() {
    let _guard =
        testing::guard("a_create_answered_by_a_server_error_after_it_was_applied_lands_once")
            .allow_slow("each round launches the real helper process twice");
    a_create_whose_write_got_no_answer_lands_once_on_the_next_push(Fault::ServerError);
}

#[test]
fn a_done_task_whose_close_was_refused_after_its_add_lands_once_and_closed() {
    let _guard =
        testing::guard("a_done_task_whose_close_was_refused_after_its_add_lands_once_and_closed")
            .allow_slow("each round launches the real helper process twice");
    for _ in 0..FRESH_HELPER_PROCESSES_TRIED {
        let s = Scenario::default();
        let mut done = task_in(1, "Ada", "Work/");
        done.completion = Completion::Done { at: None };
        s.committed(Object::Task(done));
        let (server, upstream) = serve(work_held_upstream().refusing_once("item_close"));
        let dir = tempfile::tempdir().unwrap();
        let launcher = the_real_helper_against(&server.base, dir.path());

        s.dam().push(remotes_launched_by(&launcher), None).unwrap();
        assert_eq!(s.store.unpushed(&todoist()).unwrap().len(), 1);
        let report = s
            .dam()
            .push(remotes_launched_by(&launcher), None)
            .unwrap()
            .remove(0);

        assert!(report.failed.is_empty(), "{:?}", report.failed);
        let upstream = upstream.lock().unwrap();
        let ada = upstream.item("Ada");
        assert_eq!(ada.len(), 1, "{:?}", upstream.items);
        assert_eq!(ada[0]["checked"], true, "{:?}", upstream.items);
        assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
    }
}
