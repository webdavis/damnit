use std::time::Duration;

use dam_domain::{Categories, Path};

use crate::ports::RemoteName;
use crate::secret::Secret;

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
    pub deadline: Option<ConfiguredDuration>,
    pub path: Option<Path>,
}

/// A duration next to the text it was configured as, so a message about it
/// echoes what the operator wrote rather than a normalized form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfiguredDuration {
    pub value: Duration,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialSpec {
    Literal { name: String, value: Secret },
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

impl RemoteConfig {
    /// The text after `::` in the url, handed to the helper as its second
    /// argument the way git hands one to `git-remote-<transport>`.
    pub fn address(&self) -> &str {
        self.url.split_once("::").map_or("", |(_, address)| address)
    }
}

impl Config {
    pub fn remote(&self, name: &str) -> Option<&RemoteConfig> {
        self.remotes.iter().find(|r| r.name.0 == name)
    }

    pub fn filter(&self, name: &str) -> Option<&FilterConfig> {
        self.filters.iter().find(|f| f.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "SUPERSECRETTOKEN";

    #[test]
    fn a_config_holding_a_literal_credential_never_debug_prints_it() {
        let config = Config {
            remotes: vec![RemoteConfig {
                name: RemoteName("todoist".into()),
                helper: "todoist".into(),
                url: "todoist::".into(),
                credentials: vec![CredentialSpec::Literal {
                    name: "api_token".into(),
                    value: TOKEN.into(),
                }],
                stale: None,
                deadline: None,
                path: None,
            }],
            ..Config::default()
        };
        assert!(!format!("{config:?}").contains(TOKEN));
    }

    #[test]
    fn the_address_is_the_text_after_the_helper_name() {
        let mut remote = RemoteConfig {
            name: RemoteName("gcal".into()),
            helper: "gcal".into(),
            url: "gcal::primary,team@group.calendar.google.com".into(),
            credentials: vec![],
            stale: None,
            deadline: None,
            path: None,
        };
        assert_eq!(remote.address(), "primary,team@group.calendar.google.com");
        remote.url = "todoist::".into();
        assert_eq!(remote.address(), "");
    }
}
