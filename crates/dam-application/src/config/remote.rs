use dam_domain::Path;

use super::{ConfiguredDuration, CredentialSpec, RemoteSync};
use crate::ports::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteConfig {
    pub name: RemoteName,
    pub helper: String,
    pub url: String,
    pub credentials: Vec<CredentialSpec>,
    pub deadline: Option<ConfiguredDuration>,
    pub sync: RemoteSync,
    pub path: Option<Path>,
    pub except: Vec<Path>,
}

impl RemoteConfig {
    pub const DEFAULT_DEADLINE: &str = "60s";

    pub fn address(&self) -> &str {
        self.url.split_once("::").map_or("", |(_, address)| address)
    }
}
