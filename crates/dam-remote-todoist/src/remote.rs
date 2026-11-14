use dam_helper::Remote;
use dam_protocol::{Capabilities, Mutation, MutationResult, PullResponse, PushResponse};

use crate::api::TodoistApi;
use crate::{capabilities, pull, push};

pub struct Todoist<'a> {
    pub remote: &'a str,
}

impl Remote for Todoist<'_> {
    fn capabilities(&self) -> Capabilities {
        capabilities::capabilities()
    }

    fn pull(&self, since: Option<&str>) -> Result<PullResponse, String> {
        let api = TodoistApi::from_env(self.remote)?;
        pull::pull(&api, since).map_err(|e| e.to_string())
    }

    fn push(
        &self,
        mutations: Vec<Mutation>,
        answered: &mut dyn FnMut(&MutationResult),
    ) -> Result<PushResponse, String> {
        let api = TodoistApi::from_env(self.remote)?;
        push::push(&api, mutations, answered).map_err(|e| e.to_string())
    }
}
