use super::*;
use crate::api::{Item, Project, Section, SyncResponse};
use dam_protocol::WireTask;

fn tree() -> Tree {
    Tree::from_sync(&SyncResponse {
        projects: vec![Project {
            id: "p1".into(),
            name: "Work".into(),
            parent_id: None,
            is_deleted: false,
            is_archived: false,
            inbox_project: false,
        }],
        sections: vec![Section {
            id: "s1".into(),
            name: "Now".into(),
            project_id: "p1".into(),
            is_deleted: false,
        }],
        items: vec![Item {
            id: "i1".into(),
            content: "milk".into(),
            description: String::new(),
            project_id: "p1".into(),
            section_id: Some("s1".into()),
            parent_id: None,
            priority: 1,
            due: None,
            deadline: None,
            labels: vec![],
            checked: false,
            is_deleted: false,
        }],
    })
}

fn object(path: &str, subject: &str) -> WireObject {
    WireObject {
        oid: "a".repeat(40),
        remote_id: None,
        kind: "task".into(),
        subject: subject.into(),
        body: String::new(),
        path: path.into(),
        labels: vec![],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: Some(WireTask {
            done: false,
            completed_at: None,
            priority: 4,
            due: None,
            deadline: None,
            event: None,
        }),
        event: None,
    }
}

#[test]
fn depth_zero_is_a_project() {
    assert!(matches!(
        shape_for(&tree(), &object("", "Home"), &[]).unwrap(),
        Shape::Project { parent_id: None }
    ));
}

#[test]
fn depth_one_is_a_task_unless_something_lives_or_lands_under_it_then_a_section() {
    let t = tree();
    assert!(matches!(
        shape_for(&t, &object("Work/", "Later"), &[]).unwrap(),
        Shape::Item {
            section_id: None,
            ..
        }
    ));
    assert!(matches!(
        shape_for(&t, &object("Work/", "Later"), &["Work/Later/".into()]).unwrap(),
        Shape::Section { .. }
    ));
    assert!(matches!(
        shape_for(&t, &object("Work/", "Now"), &[]).unwrap(),
        Shape::Section { .. }
    ));
}

#[test]
fn deeper_segments_name_a_section_then_parent_tasks_by_content() {
    let t = tree();
    match shape_for(&t, &object("Work/Now/", "eggs"), &[]).unwrap() {
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => {
            assert_eq!(project_id, "p1");
            assert_eq!(section_id.as_deref(), Some("s1"));
            assert!(parent_id.is_none());
        }
        other => panic!("{other:?}"),
    }
    match shape_for(&t, &object("Work/Now/milk/", "oat"), &[]).unwrap() {
        Shape::Item { parent_id, .. } => assert_eq!(parent_id.as_deref(), Some("i1")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_path_naming_no_todoist_project_is_refused_by_that_name() {
    let err = shape_for(&tree(), &object("Nope/", "x"), &[]).unwrap_err();
    assert!(err.contains("Nope"));
}
