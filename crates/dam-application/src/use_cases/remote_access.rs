use dam_domain::{Permission, Subject};

use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;

impl Dam<'_> {
    pub(crate) fn remotes_permitting(
        &self,
        permission: Permission,
        named: Option<&str>,
    ) -> Result<Vec<&RemoteConfig>, UseCaseError> {
        if let Some(name) = named
            && !self.access.permits(permission, &[Subject::Remote(name)])
        {
            return Err(Refusal::NotPermitted(permission).into());
        }
        Ok(self
            .config
            .select_remotes(named)?
            .into_iter()
            .filter(|r| self.may_reach(permission, r))
            .collect())
    }

    pub(crate) fn may_reach(&self, permission: Permission, remote: &RemoteConfig) -> bool {
        self.access
            .permits(permission, &[Subject::Remote(&remote.name.0)])
    }
}

#[cfg(test)]
mod tests;
