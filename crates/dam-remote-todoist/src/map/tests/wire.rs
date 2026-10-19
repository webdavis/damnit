use super::super::identity::RemoteIdError;
use super::super::*;
use super::build::{item, project, section};
use super::sync;
use crate::api::SyncResponse;

#[test]
fn wire_objects_carry_kind_fields_and_remote_ids() {
    let s = sync();
    let t = Tree::from_sync(&s);
    let p = project_to_wire(&t, &s.projects[1]);
    assert_eq!(p.remote_id.as_deref(), Some("p:p2"));
    assert_eq!(p.path, "Work/");
    assert_eq!(p.subject, "Client");
    assert!(p.task.as_ref().unwrap().done, "archived is done");
    let i = item_to_wire(&t, &s.items[0]).unwrap();
    assert_eq!(i.remote_id.as_deref(), Some("i:i1"));
    assert_eq!(i.subject, "milk");
    assert_eq!(i.body, "2%");
    assert_eq!(i.labels, vec!["errand".to_string()]);
    assert_eq!(i.task.as_ref().unwrap().priority, 1, "API 4 is p1");
    assert_eq!(i.task.as_ref().unwrap().due.as_deref(), Some("2026-09-25"));
    assert_eq!(i.recurrence.as_deref(), Some("every week"));
    let child = item_to_wire(&t, &s.items[1]).unwrap();
    assert!(child.task.as_ref().unwrap().done);
    assert_eq!(child.task.as_ref().unwrap().priority, 4);
}

#[test]
fn priorities_map_both_ways() {
    assert_eq!(api_priority(1), 4);
    assert_eq!(api_priority(4), 1);
    assert_eq!(dam_priority(4), 1);
    assert_eq!(dam_priority(0), 4);
    assert_eq!(dam_priority(9), 1);
    assert_eq!("i:abc".parse(), Ok(RemoteId::Item("abc".into())));
    assert_eq!("abc".parse::<RemoteId>(), Err(RemoteIdError::NoKind));
}

#[test]
fn a_remote_id_whose_tail_is_not_a_todoist_id_is_refused() {
    assert_eq!(
        "i:6X7rM8997g3RQmvh".parse(),
        Ok(RemoteId::Item("6X7rM8997g3RQmvh".into()))
    );
    assert_eq!(
        "p:c7beb07f-b226-4eb1-bf63-30d782b07b1a".parse(),
        Ok(RemoteId::Project(
            "c7beb07f-b226-4eb1-bf63-30d782b07b1a".into()
        ))
    );
    for hostile in [
        "i:../../projects/p1",
        "i:..",
        "i:1?force=true",
        "i:1/close",
        "i:1 2",
        "i:",
    ] {
        assert_eq!(
            hostile.parse::<RemoteId>(),
            Err(RemoteIdError::BadId),
            "{hostile}"
        );
    }
}

#[test]
fn a_blank_name_becomes_untitled_with_the_remote_id() {
    let s = SyncResponse {
        projects: vec![project("p1", "")],
        sections: vec![section("s1", "").under("p1")],
        items: vec![item("i1", "/").under("p1").in_section("s1")],
    };
    let t = Tree::from_sync(&s);
    assert_eq!(t.project_path("p1").as_deref(), Some("untitled-p1/"));
    assert_eq!(
        t.section_path("s1").as_deref(),
        Some("untitled-p1/untitled-s1/")
    );
}
