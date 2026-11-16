use dam_application::{MutationOp, PlannedPush, RemoteMutation, Skip};
use dam_domain::Oid;
use serde::Serialize;

use crate::view::display;

#[derive(Serialize)]
pub(super) struct MutationRow<'a> {
    pub(super) op: &'static str,
    #[serde(serialize_with = "display")]
    pub(super) oid: &'a Oid,
    pub(super) subject: Option<&'a str>,
    pub(super) container: Option<String>,
}

#[derive(Serialize)]
pub(super) struct HeldRow<'a> {
    #[serde(serialize_with = "display")]
    pub(super) oid: &'a Oid,
    pub(super) why: &'a str,
}

#[derive(Serialize)]
pub(super) struct SkippedRow<'a> {
    #[serde(serialize_with = "display")]
    pub(super) oid: &'a Oid,
    pub(super) because: &'static str,
}

impl<'a> MutationRow<'a> {
    pub(super) fn of(mutation: &'a RemoteMutation) -> MutationRow<'a> {
        MutationRow {
            op: match mutation.op {
                MutationOp::Create => "create",
                MutationOp::Update => "update",
                MutationOp::Delete => "delete",
            },
            oid: &mutation.oid,
            subject: mutation.object.as_ref().map(|o| o.base().subject.as_str()),
            container: mutation.container_remote_id.clone().or_else(|| {
                mutation
                    .container_oid
                    .as_ref()
                    .map(|o| o.short().to_string())
            }),
        }
    }
}

pub(super) fn skipped(plan: &PlannedPush) -> Vec<SkippedRow<'_>> {
    plan.skipped
        .iter()
        .map(|(oid, skip): &(Oid, Skip)| SkippedRow {
            oid,
            because: skip.as_str(),
        })
        .collect()
}
