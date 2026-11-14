mod support;

use std::os::unix::fs::PermissionsExt;

use dam_adapters::{Interrupt, ProcessLauncher};
use dam_application::RemoteTrackingRepository;
use dam_application::testing::{Scenario, config, oid, remotes_launched_by, task_in, todoist};
use dam_domain::{Completion, Object, Path};
use dam_helper::testing;
use serde_json::json;
use support::stateful_todoist::{Held, serve};

const FRESH_HELPER_PROCESSES_TRIED: usize = 8;

fn the_real_helper_against(base: &str, dir: &std::path::Path) -> ProcessLauncher {
    let wrapper = dir.join("dam-remote-todoist");
    let script = format!(
        "#!/bin/sh\nDAM_TODOIST_BASE_URL={base} exec {} \"$@\"\n",
        env!("CARGO_BIN_EXE_dam-remote-todoist")
    );
    std::fs::write(&wrapper, script).unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
    ProcessLauncher::with_search_path(dir.as_os_str(), Interrupt::default())
}

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
            .committed(Object::Task(task_in(3, "c", "Home/Now/P/")));
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

fn two_namesakes_held_upstream_with_a_done_task_under_the_first() -> Held {
    let mut held = old_p_held_upstream_with_nothing_under_it();
    held.items.push(json!({"id": "r2", "content": "P", "project_id": "home", "section_id": "now", "parent_id": null, "priority": 1, "checked": false, "is_deleted": false}));
    held.items.push(json!({"id": "t1", "content": "T", "project_id": "home", "section_id": "now", "parent_id": "r1", "priority": 1, "checked": true, "is_deleted": false}));
    held
}

#[test]
fn a_done_task_moved_out_of_the_remote_path_keeps_the_removed_namesake_upstream() {
    let _guard = testing::guard(
        "a_done_task_moved_out_of_the_remote_path_keeps_the_removed_namesake_upstream",
    )
    .allow_slow("launches the real helper process and waits on its two calls");
    let (old, namesake, done) = (oid(1), oid(2), oid(3));
    let mut scoped = config();
    scoped.remotes[0].path = Some(Path::parse("Home").unwrap());
    let s = Scenario::default().configured(scoped);
    let mut t = task_in(3, "T", "Home/Now/P/");
    t.completion = Completion::Done { at: None };
    s.committed(Object::Task(task_in(1, "P", "Home/Now/")))
        .tracked_as(&old, "i:r1")
        .committed(Object::Task(task_in(2, "P", "Home/Now/")))
        .tracked_as(&namesake, "i:r2")
        .committed(Object::Task(t.clone()))
        .tracked_as(&done, "i:t1")
        .pushed();
    t.base.path = Path::parse("Work/").unwrap();
    s.working(Object::Task(t)).deleted(&old);
    let (server, upstream) = serve(two_namesakes_held_upstream_with_a_done_task_under_the_first());
    let dir = tempfile::tempdir().unwrap();
    let launcher = the_real_helper_against(&server.base, dir.path());

    let report = s
        .dam()
        .push(remotes_launched_by(&launcher), None)
        .unwrap()
        .remove(0);

    assert_eq!(report.sent, 0, "{report:?}");
    assert_eq!(report.failed.len(), 1, "{:?}", report.failed);
    assert_eq!(report.failed[0].0, old);
    let upstream = upstream.lock().unwrap();
    assert_eq!(upstream.item("T").len(), 1, "{:?}", upstream.items);
    assert_eq!(upstream.item("P").len(), 2, "{:?}", upstream.items);
    assert!(s.store.remote_id(&todoist(), &old).unwrap().is_some());
}
