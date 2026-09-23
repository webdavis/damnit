//! The translation between dam's own vocabulary and the helper protocol.
//! Every wire type is named here and nowhere inside the application.

mod decode;
mod encode;

use dam_application::{
    IncomingObject, MutationOp, MutationOutcome, PullOutcome, RejectedObject, RemoteCapabilities,
    RemoteMutation,
};
use dam_domain::{Field, Kind, Oid};
use dam_protocol::{Capabilities, Mutation, MutationResult, PullResponse};

pub use decode::{WireError, from_wire};
pub use encode::to_wire;

/// The oid a pulled object carries until the application resolves the one it
/// tracks that remote id by. A helper sends no oid of its own.
fn placeholder_oid() -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(0))
}

/// What the application understands of a helper's declared capabilities. A
/// kind or field this dam does not know is dropped rather than refused.
pub fn capabilities_from_wire(caps: &Capabilities) -> RemoteCapabilities {
    RemoteCapabilities {
        kinds: caps.kinds.iter().filter_map(|k| Kind::parse(k)).collect(),
        fields: caps.fields.iter().filter_map(|f| Field::parse(f)).collect(),
        credentials: caps.credentials.clone(),
        incremental: caps.incremental,
    }
}

/// Reads one pull response, splitting the objects dam can use from the ones
/// it cannot. An object with no remote id is rejected under an empty name,
/// since there is nothing to track it by.
pub fn pull_from_wire(response: PullResponse) -> PullOutcome {
    let mut objects = Vec::new();
    let mut rejected = Vec::new();
    for mut wire in response.objects {
        let Some(remote_id) = wire.remote_id.take() else {
            rejected.push(RejectedObject {
                remote_id: String::new(),
                why: "the object has no remote_id".into(),
            });
            continue;
        };
        wire.oid = placeholder_oid().to_string();
        match from_wire(&wire) {
            Ok(object) => objects.push(IncomingObject { remote_id, object }),
            Err(why) => rejected.push(RejectedObject {
                remote_id,
                why: why.to_string(),
            }),
        }
    }
    PullOutcome {
        objects,
        rejected,
        removed: response.removed,
        cancelled: response.cancelled,
        sync: response.sync,
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
        object: mutation.object.as_ref().map(|o| to_wire(o, None)),
        fields: mutation
            .fields
            .iter()
            .map(|f| f.as_str().to_string())
            .collect(),
    }
}

/// A result naming an oid that is not one is dropped: the helper is an
/// untrusted subprocess and there is no change to attribute it to.
pub fn outcomes_from_wire(results: Vec<MutationResult>) -> Vec<MutationOutcome> {
    results
        .into_iter()
        .filter_map(|r| {
            Some(MutationOutcome {
                oid: Oid::parse(&r.oid).ok()?,
                ok: r.ok,
                remote_id: r.remote_id,
                why: r.why,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
