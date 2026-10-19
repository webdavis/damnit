use std::time::Duration;

use dam_domain::{Categories, Path};

use crate::errors::Refusal;
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
    pub helper: String,
    pub url: String,
    pub credentials: Vec<CredentialSpec>,
    pub stale: Option<Duration>,
    pub deadline: Option<ConfiguredDuration>,
    pub path: Option<Path>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfiguredDuration {
    pub value: Duration,
    pub text: String,
}

impl ConfiguredDuration {
    pub fn parse(text: &str) -> Option<ConfiguredDuration> {
        let mut chars = text.chars();
        let unit = chars.next_back()?;
        let n: u64 = chars.as_str().parse().ok()?;
        let secs = match unit {
            's' => Some(n),
            'm' => n.checked_mul(60),
            'h' => n.checked_mul(3600),
            _ => None,
        }?;
        Some(ConfiguredDuration {
            value: Duration::from_secs(secs),
            text: text.to_string(),
        })
    }
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
    pub fn address(&self) -> &str {
        self.url.split_once("::").map_or("", |(_, address)| address)
    }
}

impl Config {
    pub fn remote(&self, name: &str) -> Option<&RemoteConfig> {
        self.remotes.iter().find(|r| r.name.0 == name)
    }

    pub(crate) fn select_remotes(
        &self,
        remote: Option<&str>,
    ) -> Result<Vec<&RemoteConfig>, Refusal> {
        match remote {
            None => Ok(self.remotes.iter().collect()),
            Some(name) => self
                .remote(name)
                .map(|r| vec![r])
                .ok_or_else(|| Refusal::NoSuchRemote(name.to_string())),
        }
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

    fn seconds(text: &str) -> Option<u64> {
        ConfiguredDuration::parse(text).map(|d| d.value.as_secs())
    }

    #[test]
    fn a_duration_is_seconds_minutes_or_hours_and_keeps_its_text() {
        assert_eq!(seconds("30s"), Some(30));
        assert_eq!(seconds("15m"), Some(900));
        assert_eq!(seconds("2h"), Some(7200));
        assert_eq!(ConfiguredDuration::parse("2h").unwrap().text, "2h");
        assert_eq!(seconds("15"), None);
        assert_eq!(seconds("1d"), None);
    }

    #[test]
    fn a_non_ascii_empty_or_overflowing_duration_is_refused_rather_than_a_panic() {
        assert_eq!(seconds("15\u{00b5}"), None);
        assert_eq!(seconds(""), None);
        assert_eq!(seconds("\u{201c}2m\u{201d}"), None);
        assert_eq!(seconds("18446744073709551615h"), None);
    }
}
