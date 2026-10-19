use dam_helper::Remote;
use dam_protocol::{Capabilities, Mutation, PullResponse, PushResponse};

use crate::api::TodoistApi;
use crate::{capabilities, pull, push};

pub struct Todoist<'a> {
    pub remote: &'a str,
}

impl Remote for Todoist<'_> {
    fn capabilities(&self) -> Capabilities {
        capabilities::capabilities()
    }

    fn pull(&self, _since: Option<&str>) -> Result<PullResponse, String> {
        let api = TodoistApi::from_env(self.remote)?;
        pull::pull(&api).map_err(|e| e.to_string())
    }

    fn push(&self, mutations: Vec<Mutation>) -> Result<PushResponse, String> {
        let api = TodoistApi::from_env(self.remote)?;
        push::push(&api, mutations).map_err(|e| e.to_string())
    }
}
