use dam_protocol::WireObject;

use super::shape::Shape;
use crate::api::{Item, Project, Section};
use crate::map::{RemoteId, Tree, api_priority};

pub(super) fn add_created_to_tree(
    tree: &mut Tree,
    object: &WireObject,
    shape: Shape,
    id: String,
) -> String {
    match shape {
        Shape::Project { parent_id } => {
            tree.add_project(Project {
                id: id.clone(),
                name: object.subject.clone(),
                parent_id,
                is_deleted: false,
                is_archived: false,
                inbox_project: false,
            });
            RemoteId::Project(id).to_string()
        }
        Shape::Section { project_id } => {
            tree.add_section(Section {
                id: id.clone(),
                name: object.subject.clone(),
                project_id,
                is_deleted: false,
            });
            RemoteId::Section(id).to_string()
        }
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => {
            let task = object.task.as_ref();
            tree.add_item(Item {
                id: id.clone(),
                content: object.subject.clone(),
                description: object.body.clone(),
                project_id,
                section_id,
                parent_id,
                priority: task.map(|t| api_priority(t.priority)).unwrap_or(1),
                due: None,
                deadline: None,
                labels: object.labels.clone(),
                checked: task.is_some_and(|t| t.done),
                completed_at: None,
                is_deleted: false,
            });
            RemoteId::Item(id).to_string()
        }
    }
}
