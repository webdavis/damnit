use dam_protocol::{Mutation, PushResponse};

use crate::api::TodoistApi;

/// Placeholder until Task 34 applies mutations against the Todoist API.
pub fn push(_api: &TodoistApi, _mutations: Vec<Mutation>) -> Result<PushResponse, String> {
    Err("not yet".into())
}
