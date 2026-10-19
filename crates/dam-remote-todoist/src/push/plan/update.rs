use dam_protocol::{Mutation, WireObject};

use super::{Failure, Plan, numbered};
use crate::map::{RemoteId, Tree};
use crate::push::carried::{Carries, PROJECT, SECTION, fields_the_container_cannot_carry};
use crate::push::commands::{Args, Command, ItemWrite, args, command, item_args};
use crate::push::shape::{Shape, shape_for};

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
    match remote {
        RemoteId::Project(id) => {
            if changed("subject") {
                commands.push("project_update", renamed(&id, object))?;
            }
            if changed("done") {
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
                commands.push(
                    "item_move",
                    most_specific_container_move_args(tree, &id, object)?,
                )?;
            }
            let mut item = item_args(object, ItemWrite::Update { changed: &m.fields });
            if !item.is_empty() {
                item.insert("id".into(), id.clone().into());
                commands.push("item_update", item)?;
            }
            if changed("done") {
                let verb = if done {
                    "item_close"
                } else {
                    "item_uncomplete"
                };
                commands.push(verb, args([("id", id.into())]))?;
            }
        }
    }
    Ok(Plan {
        commands: commands.sent,
        creates: None,
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

fn most_specific_container_move_args(
    tree: &Tree,
    id: &str,
    object: &WireObject,
) -> Result<Args, Failure> {
    match shape_for(tree, object, &[]).map_err(Failure::OnlyThisMutation)? {
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => Ok(match (parent_id, section_id) {
            (Some(p), _) => args([("id", id.into()), ("parent_id", p.into())]),
            (None, Some(s)) => args([("id", id.into()), ("section_id", s.into())]),
            (None, None) => args([("id", id.into()), ("project_id", project_id.into())]),
        }),
        Shape::Project { .. } | Shape::Section { .. } => Err(Failure::OnlyThisMutation(
            "a path change would move the task out of task position".into(),
        )),
    }
}
