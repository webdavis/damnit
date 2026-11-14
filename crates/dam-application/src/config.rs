use std::time::Duration;

use dam_domain::{Categories, Path};

use crate::errors::Refusal;
use crate::ports::RemoteName;
use crate::secret::Secret;
use crate::tokens::Tokens;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub owner: Option<String>,
    pub done_interactive: bool,
    pub remotes: Vec<RemoteConfig>,
    pub categories: Categories,
    pub filters: Vec<FilterConfig>,
    pub tokens: Tokens,
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
        let digits = Some(chars.as_str()).filter(|d| d.bytes().all(|b| b.is_ascii_digit()))?;
        let n: u64 = digits.parse().ok()?;
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
    Computed { remote: RemoteName, name: String },
}

impl CredentialSpec {
    pub fn name(&self) -> &str {
        match self {
            CredentialSpec::Literal { name, .. }
            | CredentialSpec::Command { name, .. }
            | CredentialSpec::Env { name, .. }
            | CredentialSpec::Computed { name, .. } => name,
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
    pub fn owner_name(&self) -> &str {
        self.owner.as_deref().unwrap_or("owner")
    }

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
mod tests;
