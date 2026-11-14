mod support;

use dam_application::Notice;
use dam_application::testing::prelude::*;
use dam_application::testing::{Scenario, oid, remotes_launched_by, task_in, todoist};
use dam_domain::{Completion, Object};
use dam_helper::testing;
use support::real_helper::the_real_helper_against;
use support::stateful_todoist::undocumented::{EVERY_WAY_TODOIST_MIGHT_BEHAVE, Undocumented};
use support::stateful_todoist::{Fault, Held, serve, work_held_upstream};

const NO_ID: &str = "Todoist accepted the new task but named no id for it";
const CLOSE_REFUSED: &str =
    "Todoist accepted item_add but refused item_close: Invalid temporary id";

fn ada(done: bool) -> Object {
    let mut ada = task_in(1, "Ada", "Work/");
    if done {
        ada.completion = Completion::Done { at: None };
    }
    Object::Task(ada)
}

fn push_failed_why(s: &Scenario) -> Vec<String> {
    let notices = s.store.notices().unwrap().into_iter();
    notices
        .filter_map(|n| match n {
            Notice::PushFailed { oid: o, why, .. } if o == oid(1) => Some(why),
            _ => None,
        })
        .collect()
}

fn two_pushes(held: Held, ada: Object, still_stuck_because: Option<&str>, how: Undocumented) {
    let s = Scenario::default();
    s.committed(ada);
    s.committed(Object::Task(task_in(2, "Bob", "Work/")));
    let (server, upstream) = serve(held);
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());

    s.dam().push(remotes_launched_by(&launcher), None).unwrap();
    assert!(
        s.store.remote_id(&todoist(), &oid(2)).unwrap().is_some(),
        "{how:?}"
    );
    s.committed(Object::Task(task_in(3, "Cy", "Work/")));
    let report = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);

    let upstream = upstream.lock().unwrap();
    for name in ["Ada", "Bob", "Cy"] {
        assert_eq!(
            upstream.item(name).len(),
            1,
            "{how:?} {name}: {:?}",
            upstream.items
        );
    }
    assert!(
        s.store.remote_id(&todoist(), &oid(3)).unwrap().is_some(),
        "{how:?}"
    );
    let ada_mapped = s.store.remote_id(&todoist(), &oid(1)).unwrap().is_some();
    match still_stuck_because {
        None => {
            assert!(report.failed.is_empty(), "{how:?}: {:?}", report.failed);
            assert!(ada_mapped, "{how:?}");
        }
        Some(why) => {
            assert_eq!(report.failed, vec![(oid(1), why.to_string())], "{how:?}");
            assert_eq!(push_failed_why(&s).last().map(String::as_str), Some(why));
            assert!(!ada_mapped, "{how:?}");
            assert_eq!(s.store.unpushed(&todoist()).unwrap().len(), 1, "{how:?}");
        }
    }
}

#[test]
fn a_create_todoist_never_names_an_id_for_holds_back_only_itself() {
    let _guard = testing::guard("a_create_todoist_never_names_an_id_for_holds_back_only_itself")
        .allow_slow("each way Todoist might behave launches the real helper process twice");
    for how in EVERY_WAY_TODOIST_MIGHT_BEHAVE {
        let held = work_held_upstream()
            .behaving(how)
            .applying_the_next_write_then(Fault::HangUp);
        let stuck = (!how.replays_the_temp_id_of_a_skipped_command).then_some(NO_ID);
        two_pushes(held, ada(false), stuck, how);
    }
}

#[test]
fn a_done_task_whose_close_was_refused_never_lands_twice() {
    let _guard = testing::guard("a_done_task_whose_close_was_refused_never_lands_twice")
        .allow_slow("each way Todoist might behave launches the real helper process twice");
    for how in EVERY_WAY_TODOIST_MIGHT_BEHAVE {
        let held = work_held_upstream()
            .behaving(how)
            .refusing_once("item_close");
        let stuck = match (
            how.replays_the_temp_id_of_a_skipped_command,
            how.resolves_a_temp_id_from_an_earlier_write,
        ) {
            (true, true) => None,
            (_, false) => Some(CLOSE_REFUSED),
            (false, true) => Some(NO_ID),
        };
        two_pushes(held, ada(true), stuck, how);
    }
}
