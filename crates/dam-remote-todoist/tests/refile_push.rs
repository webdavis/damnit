mod support;

use dam_application::RemoteTrackingRepository;
use dam_application::testing::{Scenario, oid, remotes_launched_by, task_in, todoist};
use dam_domain::Object;
use dam_helper::testing;
use serde_json::{Value, json};
use support::real_helper::{FRESH_HELPER_PROCESSES_TRIED, the_real_helper_against};
use support::stateful_todoist::{Held, serve};

fn task(id: &str, content: &str, parent_id: Option<&str>) -> Value {
    json!({"id": id, "content": content, "project_id": "home", "section_id": "now", "parent_id": parent_id, "priority": 1, "checked": false, "is_deleted": false})
}

fn held_with(items: Vec<Value>) -> Held {
    Held::new(
        vec![
            json!({"id": "home", "name": "Home", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}),
        ],
        vec![json!({"id": "now", "name": "Now", "project_id": "home", "is_deleted": false})],
        items,
    )
}

fn tracked(s: &Scenario, byte: u8, subject: &str, path: &str, remote_id: &str) {
    s.committed(Object::Task(task_in(byte, subject, path)))
        .tracked_as(&oid(byte), remote_id);
}

fn pushed_through_the_real_helper(s: &Scenario, upstream: Held) -> Held {
    let (server, state) = serve(upstream);
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());
    let report = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);
    assert!(report.failed.is_empty(), "{:?}", report.failed);
    let held = state.lock().unwrap();
    Held::new(
        held.projects.clone(),
        held.sections.clone(),
        held.items.clone(),
    )
}

#[test]
fn a_child_the_removed_parent_held_moves_under_its_new_namesake_in_one_push() {
    let _guard =
        testing::guard("a_child_the_removed_parent_held_moves_under_its_new_namesake_in_one_push")
            .allow_slow("each round launches the real helper process and waits on its calls");
    for _ in 0..FRESH_HELPER_PROCESSES_TRIED {
        let s = Scenario::default();
        tracked(&s, 1, "P", "Home/Now/", "i:r1");
        tracked(&s, 3, "c", "Home/Now/P/", "i:c1");
        s.pushed();
        s.working(Object::Task(task_in(2, "P", "Home/Now/")))
            .deleted(&oid(1));

        let upstream = pushed_through_the_real_helper(
            &s,
            held_with(vec![task("r1", "P", None), task("c1", "c", Some("r1"))]),
        );

        let new_p = upstream.item("P");
        assert_eq!(new_p.len(), 1, "{:?}", upstream.items);
        assert_ne!(new_p[0]["id"], "r1");
        assert_eq!(upstream.item("c")[0]["parent_id"], new_p[0]["id"]);
        assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_none());
    }
}

#[test]
fn a_child_beside_a_namesake_already_upstream_moves_under_it_in_one_push() {
    let _guard =
        testing::guard("a_child_beside_a_namesake_already_upstream_moves_under_it_in_one_push")
            .allow_slow("each round launches the real helper process and waits on its calls");
    for _ in 0..FRESH_HELPER_PROCESSES_TRIED {
        let s = Scenario::default();
        tracked(&s, 1, "P", "Home/Now/", "i:r1");
        tracked(&s, 2, "P", "Home/Now/", "i:r2");
        tracked(&s, 3, "c", "Home/Now/P/", "i:c1");
        s.pushed();
        s.deleted(&oid(1));

        let upstream = pushed_through_the_real_helper(
            &s,
            held_with(vec![
                task("r1", "P", None),
                task("r2", "P", None),
                task("c1", "c", Some("r1")),
            ]),
        );

        let ps: Vec<&Value> = upstream.item("P");
        assert_eq!(
            (ps.len(), &ps[0]["id"]),
            (1, &json!("r2")),
            "{:?}",
            upstream.items
        );
        assert_eq!(upstream.item("c")[0]["parent_id"], "r2");
        assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_none());
    }
}
