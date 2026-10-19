use dam_protocol::{Capabilities, Mutation, PullResponse, PushResponse};

pub trait Remote {
    fn capabilities(&self) -> Capabilities;
    fn pull(&self, since: Option<&str>) -> Result<PullResponse, String>;
    fn push(&self, mutations: Vec<Mutation>) -> Result<PushResponse, String>;
}
