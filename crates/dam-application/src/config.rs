use std::time::Duration;

use dam_domain::{Categories, Path};

use crate::ports::RemoteName;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub done_interactive: bool,
    pub remotes: Vec<RemoteConfig>,
    pub categories: Categories,
    pub filters: Vec<FilterConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteConfig {
    pub name: RemoteName,
    /// The word before `::` in the url; the helper is `dam-remote-<helper>`.
    pub helper: String,
    /// The url exactly as configured, for `remote list` to echo back verbatim.
    pub url: String,
    pub credentials: Vec<CredentialSpec>,
    pub stale: Option<Duration>,
    /// How long a single helper response may take before the helper is killed.
    pub deadline: Option<Duration>,
    pub path: Option<Path>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialSpec {
    Literal { name: String, value: String },
    Command { name: String, argv: Vec<String> },
    Env { name: String, var: String },
}

impl CredentialSpec {
    pub fn name(&self) -> &str {
        match self {
            CredentialSpec::Literal { name, .. }
            | CredentialSpec::Command { name, .. }
            | CredentialSpec::Env { name, .. } => name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterConfig {
    pub name: String,
    pub query: String,
}

impl Config {
    pub fn remote(&self, name: &str) -> Option<&RemoteConfig> {
        self.remotes.iter().find(|r| r.name.0 == name)
    }

    pub fn filter(&self, name: &str) -> Option<&FilterConfig> {
        self.filters.iter().find(|f| f.name == name)
    }
}
