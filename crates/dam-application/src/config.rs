mod credential;
mod duration;
mod filter;
mod hook;
mod remote;

use std::collections::BTreeSet;

use dam_domain::Categories;

use crate::errors::Refusal;
use crate::tokens::Tokens;

pub use credential::CredentialSpec;
pub use duration::ConfiguredDuration;
pub use filter::{FilterConfig, FilterRule};
pub use hook::{HookConfig, HookEvent, OnFailure, PulledHook, STARTED_BY_HOOK_VARIABLE};
pub use remote::RemoteConfig;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub owner: Option<String>,
    pub done_interactive: Option<bool>,
    pub remotes: Vec<RemoteConfig>,
    pub categories: Categories,
    pub categories_writing_exclusive: BTreeSet<String>,
    pub filters: Vec<FilterConfig>,
    pub hooks: Vec<HookConfig>,
    pub task_pulled: Option<PulledHook>,
    pub tokens: Tokens,
}

impl Config {
    pub fn owner_name(&self) -> &str {
        self.owner.as_deref().unwrap_or("owner")
    }

    pub fn done_interactive(&self) -> bool {
        self.done_interactive.unwrap_or(false)
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

    pub fn hook(&self, event: HookEvent) -> Option<&HookConfig> {
        self.hooks.iter().find(|h| h.event == event)
    }
}

#[cfg(test)]
mod tests;
