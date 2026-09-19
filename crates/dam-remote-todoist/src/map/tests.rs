use super::*;
use crate::api::{Due, Item, Project, Section, SyncResponse};

fn sync() -> SyncResponse {
    SyncResponse {
        projects: vec![
            Project {
                id: "p1".into(),
                name: "Work".into(),
                parent_id: None,
                is_deleted: false,
                is_archived: false,
                inbox_project: false,
            },
            Project {
                id: "p2".into(),
                name: "Client".into(),
                parent_id: Some("p1".into()),
                is_deleted: false,
                is_archived: true,
                inbox_project: false,
            },
        ],
        sections: vec![Section {
            id: "s1".into(),
            name: "Now".into(),
            project_id: "p1".into(),
            is_deleted: false,
        }],
        items: vec![
            Item {
                id: "i1".into(),
                content: "milk".into(),
                description: "2%".into(),
                project_id: "p1".into(),
                section_id: Some("s1".into()),
                parent_id: None,
                priority: 4,
                due: Some(Due {
                    date: "2026-09-25".into(),
                    is_recurring: true,
                    string: "every week".into(),
                }),
                deadline: None,
                labels: vec!["errand".into()],
                checked: false,
                is_deleted: false,
            },
            Item {
                id: "i2".into(),
                content: "oat".into(),
                description: String::new(),
                project_id: "p1".into(),
                section_id: Some("s1".into()),
                parent_id: Some("i1".into()),
                priority: 1,
                due: None,
                deadline: None,
                labels: vec![],
                checked: true,
                is_deleted: false,
            },
            Item {
                id: "i3".into(),
                content: "loose".into(),
                description: String::new(),
                project_id: "p1".into(),
                section_id: None,
                parent_id: None,
                priority: 2,
                due: None,
                deadline: None,
                labels: vec![],
                checked: false,
                is_deleted: false,
            },
        ],
    }
}

#[test]
fn paths_follow_project_section_and_parent_names() {
    let t = Tree::from_sync(&sync());
    assert_eq!(t.project_path("p1").as_deref(), Some("Work/"));
    assert_eq!(t.project_path("p2").as_deref(), Some("Work/Client/"));
    assert_eq!(t.section_path("s1").as_deref(), Some("Work/Now/"));
    let items = &sync().items;
    assert_eq!(t.item_path(&items[0]).as_deref(), Some("Work/Now/"));
    assert_eq!(t.item_path(&items[1]).as_deref(), Some("Work/Now/milk/"));
    assert_eq!(t.item_path(&items[2]).as_deref(), Some("Work/"));
}

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
fn reverse_lookups_find_ids_by_path() {
    let t = Tree::from_sync(&sync());
    assert_eq!(t.project_id_for("Work/"), Some("p1"));
    assert_eq!(t.project_id_for("Work/Client/"), Some("p2"));
    assert_eq!(t.section_id_for("Work/Now/"), Some("s1"));
    assert_eq!(t.item_id_for("Work/Now/", "milk"), Some("i1"));
    assert!(t.project_id_for("Nope/").is_none());
}

#[test]
fn priorities_map_both_ways() {
    assert_eq!(api_priority(1), 4);
    assert_eq!(api_priority(4), 1);
    assert_eq!(dam_priority(4), 1);
    assert_eq!(dam_priority(0), 4);
    assert_eq!(dam_priority(9), 1);
    assert_eq!(split_remote_id("i:abc"), Some(('i', "abc")));
    assert_eq!(split_remote_id("abc"), None);
}

#[test]
fn a_project_parent_cycle_returns_none_instead_of_overflowing() {
    let s = SyncResponse {
        projects: vec![
            Project {
                id: "p1".into(),
                name: "A".into(),
                parent_id: Some("p2".into()),
                is_deleted: false,
                is_archived: false,
                inbox_project: false,
            },
            Project {
                id: "p2".into(),
                name: "B".into(),
                parent_id: Some("p1".into()),
                is_deleted: false,
                is_archived: false,
                inbox_project: false,
            },
        ],
        sections: vec![],
        items: vec![],
    };
    let t = Tree::from_sync(&s);
    assert!(t.project_path("p1").is_none());
    assert!(t.project_path("p2").is_none());
}

#[test]
fn an_item_parent_cycle_returns_none_instead_of_overflowing() {
    let s = SyncResponse {
        projects: vec![Project {
            id: "p1".into(),
            name: "Work".into(),
            parent_id: None,
            is_deleted: false,
            is_archived: false,
            inbox_project: false,
        }],
        sections: vec![],
        items: vec![
            Item {
                id: "i1".into(),
                content: "one".into(),
                description: String::new(),
                project_id: "p1".into(),
                section_id: None,
                parent_id: Some("i2".into()),
                priority: 1,
                due: None,
                deadline: None,
                labels: vec![],
                checked: false,
                is_deleted: false,
            },
            Item {
                id: "i2".into(),
                content: "two".into(),
                description: String::new(),
                project_id: "p1".into(),
                section_id: None,
                parent_id: Some("i1".into()),
                priority: 1,
                due: None,
                deadline: None,
                labels: vec![],
                checked: false,
                is_deleted: false,
            },
        ],
    };
    let t = Tree::from_sync(&s);
    assert!(t.item_path(&s.items[0]).is_none());
    assert!(item_to_wire(&t, &s.items[0]).is_none());
}
