mod shape;

use dam_protocol::{Mutation, MutationResult, PushResponse, WireObject};

use crate::api::{ApiError, Item, Project, Section, TodoistApi};
use crate::map::{Tree, api_priority, remote_id, split_remote_id};
pub use shape::{Shape, shape_for};

/// Why one mutation did not happen.
enum Failure {
    /// Reported against that mutation; the rest of the push goes on.
    Mutation(String),
    /// The push stops here. Attempting the rest would report failures Todoist
    /// never saw, and dam would mark the batch pushed on the strength of them.
    Halt(String),
}

impl From<ApiError> for Failure {
    fn from(e: ApiError) -> Failure {
        match e {
            rate @ ApiError::RateLimited { .. } => Failure::Halt(rate.to_string()),
            other => Failure::Mutation(other.to_string()),
        }
    }
}

impl From<String> for Failure {
    fn from(why: String) -> Failure {
        Failure::Mutation(why)
    }
}

impl From<&str> for Failure {
    fn from(why: &str) -> Failure {
        Failure::Mutation(why.to_string())
    }
}

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
    let mut results = Vec::with_capacity(mutations.len());
    for m in &mutations {
        match apply(api, &mut tree, m, &pending) {
            Ok(remote_id) => results.push(MutationResult {
                oid: m.oid.clone(),
                ok: true,
                remote_id,
                why: None,
            }),
            Err(Failure::Mutation(why)) => results.push(MutationResult {
                oid: m.oid.clone(),
                ok: false,
                remote_id: m.remote_id.clone(),
                why: Some(why),
            }),
            Err(Failure::Halt(why)) => return Err(why),
        }
    }
    Ok(PushResponse { results })
}

fn apply(
    api: &TodoistApi,
    tree: &mut Tree,
    m: &Mutation,
    pending: &[String],
) -> Result<Option<String>, Failure> {
    match m.op.as_str() {
        "delete" => {
            let (kind, id) = addressed(m, "delete")?;
            api.delete(&format!("/{}/{id}", plural(kind)))?;
            Ok(m.remote_id.clone())
        }
        "create" => {
            let object = m.object.as_ref().ok_or("create without an object")?;
            create(api, tree, object, pending)
        }
        "update" => {
            let object = m.object.as_ref().ok_or("update without an object")?;
            let (kind, id) = addressed(m, "update")?;
            update(api, tree, kind, id, object, &m.fields)?;
            Ok(m.remote_id.clone())
        }
        other => Err(Failure::Mutation(format!("unknown op {other:?}"))),
    }
}

/// The Todoist object a mutation names, refused by reason when the stored
/// `remote_id` is absent or is not one Todoist could have issued.
fn addressed<'m>(m: &'m Mutation, op: &str) -> Result<(char, &'m str), Failure> {
    let text = m
        .remote_id
        .as_deref()
        .ok_or_else(|| Failure::Mutation(format!("{op} without a Todoist id")))?;
    split_remote_id(text)
        .map_err(|e| Failure::Mutation(format!("{op} with an unusable Todoist id: {e}")))
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
        // `only` doubles as the create/update signal: `None` (create, every field wanted) omits
        // an absent deadline, `Some` (update, a named subset) sends `null` to clear one, since
        // Todoist documents `null` as update-only and a create has no prior deadline to clear.
        if want("deadline") {
            match (&t.deadline, only) {
                (Some(d), _) => {
                    body.insert("deadline_date".into(), d.clone().into());
                }
                (None, Some(_)) => {
                    body.insert("deadline_date".into(), serde_json::Value::Null);
                }
                (None, None) => {}
            }
        }
    }
    serde_json::Value::Object(body)
}

fn create(
    api: &TodoistApi,
    tree: &mut Tree,
    object: &WireObject,
    pending: &[String],
) -> Result<Option<String>, Failure> {
    let id_of = |v: &serde_json::Value| {
        v["id"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| Failure::Mutation("Todoist's answer has no id".to_string()))
    };
    match shape_for(tree, object, pending).map_err(Failure::Mutation)? {
        Shape::Project { parent_id } => {
            let mut body = serde_json::json!({ "name": object.subject });
            if let Some(p) = &parent_id {
                body["parent_id"] = p.clone().into();
            }
            let id = id_of(&api.post("/projects", &body)?)?;
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
            let id = id_of(&api.post("/sections", &body)?)?;
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
            let id = id_of(&api.post("/tasks", &body)?)?;
            let done = object.task.as_ref().is_some_and(|t| t.done);
            if done {
                api.post(&format!("/tasks/{id}/close"), &serde_json::json!({}))?;
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
    tree: &Tree,
    kind: char,
    id: &str,
    object: &WireObject,
    fields: &[String],
) -> Result<(), Failure> {
    let changed = |f: &str| fields.iter().any(|x| x == f);
    let done = object.task.as_ref().is_some_and(|t| t.done);
    match kind {
        'p' | 's' => {
            if changed("subject") {
                api.post(
                    &format!("/{}/{id}", plural(kind)),
                    &serde_json::json!({ "name": object.subject }),
                )?;
            }
            if kind == 'p' && changed("done") {
                let verb = if done { "archive" } else { "unarchive" };
                api.post(&format!("/projects/{id}/{verb}"), &serde_json::json!({}))?;
            }
        }
        _ => {
            if changed("path") {
                move_item(api, tree, id, object)?;
            }
            let body = task_fields(object, Some(fields));
            if body.as_object().is_some_and(|o| !o.is_empty()) {
                api.post(&format!("/tasks/{id}"), &body)?;
            }
            if changed("done") {
                let verb = if done { "close" } else { "reopen" };
                api.post(&format!("/tasks/{id}/{verb}"), &serde_json::json!({}))?;
            }
        }
    }
    Ok(())
}

/// Todoist's move endpoint takes exactly one of project_id, section_id or parent_id;
/// the most specific container the new path resolves to is the one sent.
fn move_item(api: &TodoistApi, tree: &Tree, id: &str, object: &WireObject) -> Result<(), Failure> {
    let body = match shape_for(tree, object, &[]).map_err(Failure::Mutation)? {
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => match (parent_id, section_id) {
            (Some(p), _) => serde_json::json!({ "parent_id": p }),
            (None, Some(s)) => serde_json::json!({ "section_id": s }),
            (None, None) => serde_json::json!({ "project_id": project_id }),
        },
        _ => {
            return Err(Failure::Mutation(
                "a path change would move the task out of task position".into(),
            ));
        }
    };
    api.post(&format!("/tasks/{id}/move"), &body)?;
    Ok(())
}
