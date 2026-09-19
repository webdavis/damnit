use dam_protocol::PullResponse;

use crate::api::TodoistApi;

/// Placeholder until Task 33 maps a synced Todoist snapshot into `PullResponse`.
pub fn pull(_api: &TodoistApi) -> Result<PullResponse, String> {
    Err("not yet".into())
}
