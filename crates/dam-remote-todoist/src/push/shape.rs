use dam_protocol::WireObject;

use crate::map::{RemoteId, Tree};
use several::several;

#[derive(Debug)]
pub enum Shape {
    Project { parent_id: Option<String> },
    Section { project_id: String },
    Item(ItemPlace),
}

#[derive(Debug)]
pub struct ItemPlace {
    pub project_id: String,
    pub section_id: Option<String>,
    pub parent_id: Option<String>,
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
        .map_err(|n| several(n, "projects", project_name, "", object))?
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
    task_under_parent_tasks_by_content(tree, object, &segments, project_path, project_id)
}

pub fn shape_inside(
    tree: &Tree,
    object: &WireObject,
    containers_filled_this_push: &[String],
    container: &RemoteId,
) -> Result<Shape, String> {
    let place = place_inside(tree, container)?;
    Ok(match container {
        RemoteId::Project(_) => section_if_anything_lives_or_lands_under_it(
            tree,
            object,
            containers_filled_this_push,
            place.project_id,
        ),
        RemoteId::Section(_) | RemoteId::Item(_) => Shape::Item(place),
    })
}

pub fn place_inside(tree: &Tree, container: &RemoteId) -> Result<ItemPlace, String> {
    let (project_id, section_id) = tree.project_and_section_of(container).ok_or_else(|| {
        format!("Todoist no longer has {container}; run: dam pull, then dam push")
    })?;
    let parent_id = match container {
        RemoteId::Item(id) => Some(id.clone()),
        RemoteId::Section(_) | RemoteId::Project(_) => None,
    };
    Ok(ItemPlace {
        project_id,
        section_id,
        parent_id,
    })
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
    Shape::Item(ItemPlace {
        project_id,
        section_id: None,
        parent_id: None,
    })
}

fn task_under_parent_tasks_by_content(
    tree: &Tree,
    object: &WireObject,
    segments: &[&str],
    project_path: String,
    project_id: String,
) -> Result<Shape, String> {
    let section_path = format!("{}{}/", project_path, segments[1]);
    let section_id = tree
        .section_id_for(&section_path)
        .map_err(|n| several(n, "sections", segments[1], &project_path, object))?
        .map(str::to_string);
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
            .map_err(|n| several(n, "tasks", segment, &walked, object))?
            .ok_or_else(|| format!("no Todoist task named {segment:?} under {walked}"))?;
        parent_id = Some(id.to_string());
        walked = format!("{walked}{segment}/");
    }
    Ok(Shape::Item(ItemPlace {
        project_id,
        section_id,
        parent_id,
    }))
}

mod several;

#[cfg(test)]
mod tests;
