use std::collections::BTreeMap;

use dam_application::{IncomingObject, PullOutcome, RejectedObject};
use dam_domain::Oid;
use dam_protocol::PullResponse;

use super::from_wire;

const NO_REMOTE_ID: &str = "an object with no remote id was skipped";

fn placeholder_oid_until_the_application_resolves_the_remote_id() -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(0))
}

pub fn pull_from_wire(response: PullResponse) -> PullOutcome {
    let mut objects = Vec::new();
    let mut container_remote_ids = BTreeMap::new();
    let mut rejected = Vec::new();
    for mut wire in response.objects {
        let Some(remote_id) = wire.remote_id.take() else {
            rejected.push(RejectedObject {
                remote_id: String::new(),
                why: NO_REMOTE_ID.into(),
            });
            continue;
        };
        wire.oid = placeholder_oid_until_the_application_resolves_the_remote_id().to_string();
        match from_wire(&wire) {
            Ok(object) => {
                if let Some(container) = wire.container_remote_id {
                    container_remote_ids.insert(remote_id.clone(), container);
                }
                objects.push(IncomingObject { remote_id, object })
            }
            Err(why) => rejected.push(RejectedObject {
                remote_id,
                why: why.to_string(),
            }),
        }
    }
    rejected.extend(
        response
            .rejected
            .into_iter()
            .map(|rejection| RejectedObject {
                why: if rejection.remote_id.is_empty() {
                    NO_REMOTE_ID.into()
                } else {
                    rejection.why
                },
                remote_id: rejection.remote_id,
            }),
    );
    PullOutcome {
        objects,
        container_remote_ids,
        rejected,
        removed: response.removed,
        cancelled: response.cancelled,
        sync: response.sync,
        complete: response.complete,
    }
}
