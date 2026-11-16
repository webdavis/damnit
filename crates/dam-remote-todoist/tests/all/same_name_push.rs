use crate::support;

use dam_application::RemoteTrackingRepository;
use dam_application::testing::{Scenario, oid, remotes_launched_by, task_in, todoist};
use dam_domain::{Object, Oid, Task};
use dam_helper::testing;
use serde_json::json;
use support::real_helper::{FRESH_HELPER_PROCESSES_TRIED, the_real_helper_against};
use support::stateful_todoist::{Held, serve};

fn old_p_held_upstream_with_nothing_under_it() -> Held {
    Held::new(
        vec![
            json!({"id": "home", "name": "Home", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}),
        ],
        vec![json!({"id": "now", "name": "Now", "project_id": "home", "is_deleted": false})],
        vec![
            json!({"id": "r1", "content": "P", "project_id": "home", "section_id": "now", "parent_id": null, "priority": 1, "checked": false, "is_deleted": false}),
        ],
    )
}

fn child_of(byte: u8, subject: &str, path: &str, parent: &Oid) -> Task {
    let mut child = task_in(byte, subject, path);
    child.base.parent = Some(parent.clone());
    child
}

#[test]
fn a_child_under_a_new_namesake_of_a_removed_parent_lands_under_the_new_one_and_survives() {
    let _guard = testing::guard(
        "a_child_under_a_new_namesake_of_a_removed_parent_lands_under_the_new_one_and_survives",
    )
    .allow_slow("each round launches the real helper process and waits on its two calls");
    for _ in 0..FRESH_HELPER_PROCESSES_TRIED {
        let (old, new, child) = (oid(1), oid(2), oid(3));
        let s = Scenario::default();
        s.committed(Object::Task(task_in(1, "P", "Home/Now/")))
            .tracked_as(&old, "i:r1")
            .pushed();
        s.deleted(&old)
            .working(Object::Task(task_in(2, "P", "Home/Now/")))
            .committed(Object::Task(child_of(3, "c", "Home/Now/P/", &new)));
        let (server, upstream) = serve(old_p_held_upstream_with_nothing_under_it());
        let dir = tempfile::tempdir().unwrap();
        let launcher = the_real_helper_against(&server.base, dir.path());

        let report = s
            .dam()
            .push(remotes_launched_by(&launcher), None)
            .unwrap()
            .remove(0);

        assert!(report.failed.is_empty(), "{:?}", report.failed);
        let upstream = upstream.lock().unwrap();
        let new_p = upstream.item("P");
        let kid = upstream.item("c");
        assert_eq!((new_p.len(), kid.len()), (1, 1), "{:?}", upstream.items);
        assert_ne!(new_p[0]["id"], "r1");
        assert_eq!(kid[0]["parent_id"], new_p[0]["id"]);
        let store = &s.store;
        assert!(store.remote_id(&todoist(), &old).unwrap().is_none());
        assert!(store.remote_id(&todoist(), &new).unwrap().is_some());
        assert!(store.remote_id(&todoist(), &child).unwrap().is_some());
    }
}
