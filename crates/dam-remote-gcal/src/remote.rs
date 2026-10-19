use dam_helper::Remote;
use dam_protocol::{Capabilities, Mutation, PullResponse, PushResponse};

use crate::access::access_token;
use crate::address::calendars;
use crate::calendar_api::CalendarApi;
use crate::{Credentials, Endpoints, capabilities, pull, push};
use dam_helper::agent;

pub struct Gcal<'a> {
    pub remote: &'a str,
    pub address: &'a str,
}

impl Remote for Gcal<'_> {
    fn capabilities(&self) -> Capabilities {
        capabilities::capabilities()
    }

    fn pull(&self, since: Option<&str>) -> Result<PullResponse, String> {
        let credentials = Credentials::from_env(self.remote)?;
        let endpoints = Endpoints::from_env()?;
        let agent = agent();
        let access = access_token(&agent, &endpoints, &credentials).map_err(|e| e.to_string())?;
        let api = CalendarApi::new(agent, &endpoints, access);
        pull::pull(
            &api,
            &calendars(self.address),
            since,
            jiff::Timestamp::now(),
        )
        .map_err(|e| e.to_string())
    }

    fn push(&self, mutations: Vec<Mutation>) -> Result<PushResponse, String> {
        Ok(push::refuse(&mutations))
    }
}
