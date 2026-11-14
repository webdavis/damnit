use dam_protocol::{Capabilities, Mutation, MutationResult, PullResponse, PushResponse};

pub trait Remote {
    fn capabilities(&self) -> Capabilities;
    fn pull(&self, since: Option<&str>) -> Result<PullResponse, String>;
    fn push(
        &self,
        mutations: Vec<Mutation>,
        answered: &mut dyn FnMut(&MutationResult),
    ) -> Result<PushResponse, String>;
}
