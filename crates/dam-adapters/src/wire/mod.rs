mod declared;
mod decode;
mod encode;
mod pull;

use dam_application::{
    HelperError, MutationOp, MutationOutcome, MutationVerdict, PushOutcome, RemoteCapabilities,
    RemoteMutation,
};
use dam_domain::{Field, Kind, Object, Oid};
use dam_protocol::{Capabilities, Mutation, PushResponse, Verdict, WireObject};

pub use declared::declared_on_the_wire;
pub use decode::from_wire;
pub use encode::{event_to_wire, to_wire};
pub use pull::pull_from_wire;

pub fn capabilities_from_wire(caps: &Capabilities) -> RemoteCapabilities {
    RemoteCapabilities {
        kinds: caps.kinds.iter().filter_map(|k| Kind::parse(k)).collect(),
        fields: caps.fields.iter().filter_map(|f| Field::parse(f)).collect(),
        credentials: caps.credentials.clone(),
        incremental: caps.incremental,
        removes_only_empty_containers: caps.removes_only_empty_containers,
        answers_login: caps.answers_login,
    }
}

pub fn mutation_to_wire(mutation: RemoteMutation) -> Mutation {
    Mutation {
        op: match mutation.op {
            MutationOp::Create => "create",
            MutationOp::Update => "update",
            MutationOp::Delete => "delete",
        }
        .to_string(),
        oid: mutation.oid.to_string(),
        idempotency_key: mutation.idempotency_key,
        remote_id: mutation.remote_id,
        container_remote_id: mutation.container_remote_id,
        object: mutation.object.as_ref().map(without_local_fields),
        fields: mutation
            .fields
            .iter()
            .map(|f| f.as_str().to_string())
            .collect(),
    }
}

fn without_local_fields(object: &Object) -> WireObject {
    let mut wire = to_wire(object, None);
    wire.parent = None;
    if let Some(task) = &mut wire.task {
        task.focus = None;
    }
    wire
}

pub fn push_from_wire(response: PushResponse) -> PushOutcome {
    let outcomes = response
        .results
        .into_iter()
        .filter_map(|r| {
            Some(MutationOutcome {
                oid: Oid::parse(&r.oid).ok()?,
                verdict: match r.verdict {
                    Verdict::Accepted { remote_id } => MutationVerdict::Accepted { remote_id },
                    Verdict::Refused { why, unasked } => MutationVerdict::Refused { why, unasked },
                    Verdict::Unanswered { why } => MutationVerdict::Unanswered { why },
                },
            })
        })
        .collect();
    PushOutcome {
        outcomes,
        stopped_by: response.stopped.map(HelperError::Remote),
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod local_field_tests;

#[cfg(test)]
mod pull_tests;

#[cfg(test)]
mod rejection_tests;

#[cfg(test)]
mod vocabulary_tests;

#[cfg(test)]
mod verdict_tests;

#[cfg(test)]
mod declared_tests;

#[cfg(test)]
mod event_detail_tests;
