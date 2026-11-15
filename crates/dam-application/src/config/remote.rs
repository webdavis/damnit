use std::time::Duration;

use dam_domain::Path;

use super::{ConfiguredDuration, CredentialSpec};
use crate::ports::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteConfig {
    pub name: RemoteName,
    pub helper: String,
    pub url: String,
    pub credentials: Vec<CredentialSpec>,
    pub deadline: Option<ConfiguredDuration>,
    pub sync_every: Option<ConfiguredDuration>,
    pub path: Option<Path>,
}

impl RemoteConfig {
    pub const DEFAULT_DEADLINE: &str = "60s";
    pub const SHORTEST_SYNC_EVERY: Duration = Duration::from_secs(60);
    pub const SHORTEST_SYNC_EVERY_TEXT: &str = "1m";

    pub fn address(&self) -> &str {
        self.url.split_once("::").map_or("", |(_, address)| address)
    }
}
