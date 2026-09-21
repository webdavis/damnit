use dam_protocol::WireObject;

use crate::map::Tree;

/// What a dam object at this path becomes on Todoist.
#[derive(Debug)]
pub enum Shape {
    Project {
        parent_id: Option<String>,
    },
    Section {
        project_id: String,
    },
    Item {
        project_id: String,
        section_id: Option<String>,
        parent_id: Option<String>,
    },
}

/// Depth 0 is a project. Depth 1 is a section when something already lives (or is about to
/// land, this push) under `<path><subject>/`, otherwise a task in the project. Depth 2+ resolves
/// each further segment to a parent item by content.
pub fn shape_for(
    tree: &Tree,
    object: &WireObject,
    pending_paths: &[String],
) -> Result<Shape, String> {
    let segments: Vec<&str> = object.path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Ok(Shape::Project { parent_id: None });
    }
    let project_path = format!("{}/", segments[0]);
    let project_id = tree
        .project_id_for(&project_path)
        .ok_or_else(|| format!("no Todoist project named {:?}", segments[0]))?
        .to_string();
    if segments.len() == 1 {
        let own = format!("{}{}/", object.path, object.subject);
        if tree.has_children_at(&own) || pending_paths.contains(&own) {
            return Ok(Shape::Section { project_id });
        }
        return Ok(Shape::Item {
            project_id,
            section_id: None,
            parent_id: None,
        });
    }
    let section_path = format!("{}{}/", project_path, segments[1]);
    let section_id = tree.section_id_for(&section_path).map(str::to_string);
    let mut walked = if section_id.is_some() {
        section_path.clone()
    } else {
        project_path.clone()
    };
    let first_item = if section_id.is_some() { 2 } else { 1 };
    let mut parent_id = None;
    for segment in &segments[first_item..] {
        let id = tree
            .item_id_for(&walked, segment)
            .ok_or_else(|| format!("no Todoist task named {segment:?} under {walked}"))?;
        parent_id = Some(id.to_string());
        walked = format!("{walked}{segment}/");
    }
    Ok(Shape::Item {
        project_id,
        section_id,
        parent_id,
    })
}

#[cfg(test)]
mod tests {
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
    fn depth_decides_the_shape() {
        let t = tree();
        assert!(matches!(
            shape_for(&t, &object("", "Home"), &[]).unwrap(),
            Shape::Project { parent_id: None }
        ));
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
        let err = shape_for(&t, &object("Nope/", "x"), &[]).unwrap_err();
        assert!(err.contains("Nope"));
    }
}
