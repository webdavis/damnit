mod credential;
mod duration;
mod filter;
mod remote;

use dam_domain::Categories;

use crate::errors::Refusal;
use crate::tokens::Tokens;

pub use credential::CredentialSpec;
pub use duration::ConfiguredDuration;
pub use filter::FilterConfig;
pub use remote::RemoteConfig;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub owner: Option<String>,
    pub done_interactive: bool,
    pub remotes: Vec<RemoteConfig>,
    pub categories: Categories,
    pub filters: Vec<FilterConfig>,
    pub tokens: Tokens,
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
