use dam_domain::Path;

use super::{ConfiguredDuration, CredentialSpec};
use crate::ports::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteConfig {
    pub name: RemoteName,
    pub helper: String,
    pub url: String,
    pub credentials: Vec<CredentialSpec>,
    pub stale: Option<ConfiguredDuration>,
    pub deadline: Option<ConfiguredDuration>,
    pub path: Option<Path>,
}

impl RemoteConfig {
    pub const DEFAULT_DEADLINE: &str = "60s";

    pub fn address(&self) -> &str {
        self.url.split_once("::").map_or("", |(_, address)| address)
    }
}
