use crate::config::RemoteConfig;
use crate::credentials::resolve_credentials;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::RemoteHelper;
use crate::remote::RemoteCapabilities;
use crate::service::{Dam, Remotes};

impl Dam<'_> {
    pub(crate) fn connect(
        &self,
        remotes: Remotes<'_>,
        remote: &RemoteConfig,
    ) -> Result<(Box<dyn RemoteHelper>, RemoteCapabilities), UseCaseError> {
        let progress = self.shown_progress();
        let configured: Vec<String> = remote
            .credentials
            .iter()
            .map(|spec| spec.name().to_string())
            .collect();
        let resolved = resolve_credentials(remotes.credentials, remote, &configured, progress)
            .map_err(|e| match self.access.token() {
                Some(_) => e.withholding_what_credential_code_raised(),
                None => e,
            })?;
        let mut helper = remotes.launcher.launch(remote, &resolved)?;
        let caps = helper.capabilities()?;
        if let Some(name) = declared_by_the_helper_but_not_configured(&caps, &configured) {
            return Err(Refusal::MissingCredential {
                remote: remote.name.0.clone(),
                name: name.clone(),
            }
            .into());
        }
        self.store.set_accepted_kinds(&remote.name, &caps.kinds)?;
        self.store.set_accepted_fields(&remote.name, &caps.fields)?;
        Ok((helper, caps))
    }
}

fn declared_by_the_helper_but_not_configured<'a>(
    caps: &'a RemoteCapabilities,
    configured: &[String],
) -> Option<&'a String> {
    caps.credentials
        .iter()
        .find(|name| !configured.contains(name))
}
