use dam_protocol::{Mutation, MutationResult, PushResponse};

pub const READ_ONLY: &str = "Google Calendar is read-only through dam-remote-gcal: it pulls events \
and never creates, changes or deletes one, so this change stays in dam";

pub fn refuse(mutations: &[Mutation]) -> PushResponse {
    PushResponse {
        results: mutations
            .iter()
            .map(|m| MutationResult {
                oid: m.oid.clone(),
                ok: false,
                remote_id: None,
                why: Some(READ_ONLY.into()),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests;
