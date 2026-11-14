use dam_protocol::{Mutation, WireObject};

use super::{Failure, Plan, TreeChange, numbered};
use crate::map::{RemoteId, Tree};
use crate::push::carried::{Carries, PROJECT, SECTION, fields_the_container_cannot_carry};
use crate::push::commands::{Args, Command, ItemWrite, args, command, item_args};
use crate::push::shape::{ItemPlace, Shape, place_inside, shape_for};

struct Commands<'m> {
    m: &'m Mutation,
    sent: Vec<Command>,
}

impl Commands<'_> {
    fn push(&mut self, kind: &'static str, args: Args) -> Result<(), Failure> {
        let uuid = numbered(self.m, self.sent.len())?;
        self.sent.push(command(kind, uuid, args));
        Ok(())
    }
}

pub fn update_plan(
    tree: &Tree,
    m: &Mutation,
    object: &WireObject,
    remote: RemoteId,
) -> Result<Plan, Failure> {
    let changed = |f: &str| m.fields.iter().any(|x| x == f);
    let done = object.task.as_ref().is_some_and(|t| t.done);
    let refused = match &remote {
        RemoteId::Project(_) => container_refusal(tree, m, object, &remote, &PROJECT),
        RemoteId::Section(_) => container_refusal(tree, m, object, &remote, &SECTION),
        RemoteId::Item(_) => None,
    };
    let mut commands = Commands { m, sent: vec![] };
    let mut changes = vec![];
    let rename = changed("subject").then(|| TreeChange::Renames {
        renamed: remote.clone(),
        name: object.subject.clone(),
    });
    match remote {
        RemoteId::Project(id) => {
            if changed("subject") {
                commands.push("project_update", renamed(&id, object))?;
            }
            if changed("done") && (done || !tree.shows_project_unarchived(&id)) {
                let verb = if done {
                    "project_archive"
                } else {
                    "project_unarchive"
                };
                commands.push(verb, args([("id", id.into())]))?;
            }
        }
        RemoteId::Section(id) => {
            if changed("subject") {
                commands.push("section_update", renamed(&id, object))?;
            }
        }
        RemoteId::Item(id) => {
            if changed("path") {
                let to = item_place(tree, m, object)?;
                commands.push("item_move", most_specific_container_move_args(&id, &to))?;
                changes.push(TreeChange::MovesItem { id: id.clone(), to });
            }
            let mut item = item_args(object, ItemWrite::Update { changed: &m.fields });
            if !item.is_empty() {
                item.insert("id".into(), id.clone().into());
                commands.push("item_update", item)?;
            }
            if changed("done") && (done || !tree.shows_item_open(&id)) {
                let verb = if done {
                    "item_close"
                } else {
                    "item_uncomplete"
                };
                commands.push(verb, args([("id", id.into())]))?;
            }
        }
    }
    changes.extend(rename);
    Ok(Plan {
        commands: commands.sent,
        changes,
        refused,
    })
}

fn container_refusal(
    tree: &Tree,
    m: &Mutation,
    object: &WireObject,
    remote: &RemoteId,
    carries: &Carries,
) -> Option<String> {
    let held = tree.held_wire(remote);
    fields_the_container_cannot_carry(carries, &m.fields, object, held.as_ref())
}

fn renamed(id: &str, object: &WireObject) -> Args {
    args([("id", id.into()), ("name", object.subject.clone().into())])
}

fn item_place(tree: &Tree, m: &Mutation, object: &WireObject) -> Result<ItemPlace, Failure> {
    if let Some(named) = &m.container_remote_id {
        let container: RemoteId = named.parse().map_err(|e| {
            Failure::RefusedWithoutAVerdict(format!("an unusable Todoist container id: {e}"))
        })?;
        return place_inside(tree, &container).map_err(Failure::RefusedWithoutAVerdict);
    }
    match shape_for(tree, object, &[]).map_err(Failure::RefusedWithoutAVerdict)? {
        Shape::Item(place) => Ok(place),
        Shape::Project { .. } | Shape::Section { .. } => Err(Failure::RefusedWithoutAVerdict(
            "a path change would move the task out of task position".into(),
        )),
    }
}

fn most_specific_container_move_args(id: &str, to: &ItemPlace) -> Args {
    let (key, container) = match (&to.parent_id, &to.section_id) {
        (Some(p), _) => ("parent_id", p),
        (None, Some(s)) => ("section_id", s),
        (None, None) => ("project_id", &to.project_id),
    };
    args([("id", id.into()), (key, container.clone().into())])
}
