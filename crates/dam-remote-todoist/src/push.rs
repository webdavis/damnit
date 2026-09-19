mod shape;

use dam_protocol::{Mutation, MutationResult, PushResponse, WireObject};

use crate::api::{Item, Project, Section, TodoistApi};
use crate::map::{Tree, api_priority, remote_id, split_remote_id};
pub use shape::{Shape, shape_for};

pub fn push(api: &TodoistApi, mutations: Vec<Mutation>) -> Result<PushResponse, String> {
    let sync = api.sync_all().map_err(|e| e.to_string())?;
    let mut tree = Tree::from_sync(&sync);
    // Each object's own `path` names the container it lives directly under; collecting
    // those (not each object's own subject-appended location) is what lets a depth-1
    // create become a section when one of its children arrives later in the same push.
    let pending: Vec<String> = mutations
        .iter()
        .filter_map(|m| m.object.as_ref())
        .map(|o| o.path.clone())
        .collect();
    let results = mutations
        .iter()
        .map(|m| match apply(api, &mut tree, m, &pending) {
            Ok(remote_id) => MutationResult {
                oid: m.oid.clone(),
                ok: true,
                remote_id,
                why: None,
            },
            Err(why) => MutationResult {
                oid: m.oid.clone(),
                ok: false,
                remote_id: m.remote_id.clone(),
                why: Some(why),
            },
        })
        .collect();
    Ok(PushResponse { results })
}

fn apply(
    api: &TodoistApi,
    tree: &mut Tree,
    m: &Mutation,
    pending: &[String],
) -> Result<Option<String>, String> {
    match m.op.as_str() {
        "delete" => {
            let (kind, id) = m
                .remote_id
                .as_deref()
                .and_then(split_remote_id)
                .ok_or("delete without a Todoist id")?;
            api.delete(&format!("/{}/{id}", plural(kind)))
                .map_err(|e| e.to_string())?;
            Ok(m.remote_id.clone())
        }
        "create" => {
            let object = m.object.as_ref().ok_or("create without an object")?;
            create(api, tree, object, pending)
        }
        "update" => {
            let object = m.object.as_ref().ok_or("update without an object")?;
            let (kind, id) = m
                .remote_id
                .as_deref()
                .and_then(split_remote_id)
                .ok_or("update without a Todoist id")?;
            update(api, kind, id, object, &m.fields)?;
            Ok(m.remote_id.clone())
        }
        other => Err(format!("unknown op {other:?}")),
    }
}

fn plural(kind: char) -> &'static str {
    match kind {
        'p' => "projects",
        's' => "sections",
        _ => "tasks",
    }
}

fn task_fields(object: &WireObject, only: Option<&[String]>) -> serde_json::Value {
    let want = |f: &str| only.is_none_or(|fields| fields.iter().any(|x| x == f));
    let mut body = serde_json::Map::new();
    if want("subject") {
        body.insert("content".into(), object.subject.clone().into());
    }
    if want("body") {
        body.insert("description".into(), object.body.clone().into());
    }
    if want("labels") {
        body.insert("labels".into(), object.labels.clone().into());
    }
    if let Some(t) = &object.task {
        if want("priority") {
            body.insert("priority".into(), api_priority(t.priority).into());
        }
        if want("due") || want("recurrence") {
            match (&object.recurrence, &t.due) {
                (Some(rule), _) => {
                    body.insert("due_string".into(), rule.clone().into());
                }
                (None, Some(d)) => {
                    body.insert("due_date".into(), d.clone().into());
                }
                (None, None) => {
                    body.insert(
                        "due_string".into(),
                        serde_json::Value::String("no date".into()),
                    );
                }
            }
        }
        if want("deadline")
            && let Some(d) = &t.deadline
        {
            body.insert("deadline_date".into(), d.clone().into());
        }
    }
    serde_json::Value::Object(body)
}

fn create(
    api: &TodoistApi,
    tree: &mut Tree,
    object: &WireObject,
    pending: &[String],
) -> Result<Option<String>, String> {
    let id_of = |v: &serde_json::Value| {
        v["id"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "Todoist's answer has no id".to_string())
    };
    match shape_for(tree, object, pending)? {
        Shape::Project { parent_id } => {
            let mut body = serde_json::json!({ "name": object.subject });
            if let Some(p) = &parent_id {
                body["parent_id"] = p.clone().into();
            }
            let id = id_of(&api.post("/projects", &body).map_err(|e| e.to_string())?)?;
            tree.add_project(Project {
                id: id.clone(),
                name: object.subject.clone(),
                parent_id,
                is_deleted: false,
                is_archived: false,
                inbox_project: false,
            });
            Ok(Some(remote_id('p', &id)))
        }
        Shape::Section { project_id } => {
            let body = serde_json::json!({ "name": object.subject, "project_id": project_id });
            let id = id_of(&api.post("/sections", &body).map_err(|e| e.to_string())?)?;
            tree.add_section(Section {
                id: id.clone(),
                name: object.subject.clone(),
                project_id,
                is_deleted: false,
            });
            Ok(Some(remote_id('s', &id)))
        }
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => {
            let mut body = task_fields(object, None);
            body["project_id"] = project_id.clone().into();
            if let Some(s) = &section_id {
                body["section_id"] = s.clone().into();
            }
            if let Some(p) = &parent_id {
                body["parent_id"] = p.clone().into();
            }
            let id = id_of(&api.post("/tasks", &body).map_err(|e| e.to_string())?)?;
            let done = object.task.as_ref().is_some_and(|t| t.done);
            if done {
                api.post(&format!("/tasks/{id}/close"), &serde_json::json!({}))
                    .map_err(|e| e.to_string())?;
            }
            tree.add_item(Item {
                id: id.clone(),
                content: object.subject.clone(),
                description: object.body.clone(),
                project_id,
                section_id,
                parent_id,
                priority: object
                    .task
                    .as_ref()
                    .map(|t| api_priority(t.priority))
                    .unwrap_or(1),
                due: None,
                deadline: None,
                labels: object.labels.clone(),
                checked: done,
                is_deleted: false,
            });
            Ok(Some(remote_id('i', &id)))
        }
    }
}

fn update(
    api: &TodoistApi,
    kind: char,
    id: &str,
    object: &WireObject,
    fields: &[String],
) -> Result<(), String> {
    let changed = |f: &str| fields.iter().any(|x| x == f);
    let done = object.task.as_ref().is_some_and(|t| t.done);
    match kind {
        'p' | 's' => {
            if changed("subject") {
                api.post(
                    &format!("/{}/{id}", plural(kind)),
                    &serde_json::json!({ "name": object.subject }),
                )
                .map_err(|e| e.to_string())?;
            }
            if kind == 'p' && changed("done") {
                let verb = if done { "archive" } else { "unarchive" };
                api.post(&format!("/projects/{id}/{verb}"), &serde_json::json!({}))
                    .map_err(|e| e.to_string())?;
            }
        }
        _ => {
            let body = task_fields(object, Some(fields));
            if body.as_object().is_some_and(|o| !o.is_empty()) {
                api.post(&format!("/tasks/{id}"), &body)
                    .map_err(|e| e.to_string())?;
            }
            if changed("done") {
                let verb = if done { "close" } else { "reopen" };
                api.post(&format!("/tasks/{id}/{verb}"), &serde_json::json!({}))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
