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
mod tests;
