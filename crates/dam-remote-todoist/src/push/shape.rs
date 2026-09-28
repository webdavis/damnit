use dam_protocol::WireObject;

use crate::map::Tree;

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

pub fn shape_for(
    tree: &Tree,
    object: &WireObject,
    containers_filled_this_push: &[String],
) -> Result<Shape, String> {
    let segments: Vec<&str> = object.path.split('/').filter(|s| !s.is_empty()).collect();
    let Some(project_name) = segments.first() else {
        return Ok(Shape::Project { parent_id: None });
    };
    let project_path = format!("{project_name}/");
    let project_id = tree
        .project_id_for(&project_path)
        .ok_or_else(|| format!("no Todoist project named {project_name:?}"))?
        .to_string();
    if segments.len() == 1 {
        return Ok(section_if_anything_lives_or_lands_under_it(
            tree,
            object,
            containers_filled_this_push,
            project_id,
        ));
    }
    task_under_parent_tasks_by_content(tree, &segments, project_path, project_id)
}

fn section_if_anything_lives_or_lands_under_it(
    tree: &Tree,
    object: &WireObject,
    containers_filled_this_push: &[String],
    project_id: String,
) -> Shape {
    let own = format!("{}{}/", object.path, object.subject);
    if tree.has_children_at(&own) || containers_filled_this_push.contains(&own) {
        return Shape::Section { project_id };
    }
    Shape::Item {
        project_id,
        section_id: None,
        parent_id: None,
    }
}

fn task_under_parent_tasks_by_content(
    tree: &Tree,
    segments: &[&str],
    project_path: String,
    project_id: String,
) -> Result<Shape, String> {
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
}
