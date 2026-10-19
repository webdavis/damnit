use crate::config::RemoteConfig;
use crate::credentials::resolve_credentials;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{AcceptedKindsRepository, RemoteHelper};
use crate::remote::RemoteCapabilities;
use crate::service::Remotes;

pub(crate) fn connect(
    remotes: Remotes<'_>,
    accepted_kinds: &dyn AcceptedKindsRepository,
    remote: &RemoteConfig,
) -> Result<(Box<dyn RemoteHelper>, RemoteCapabilities), UseCaseError> {
    let configured: Vec<String> = remote
        .credentials
        .iter()
        .map(|spec| spec.name().to_string())
        .collect();
    let resolved = resolve_credentials(remotes.credentials, remote, &configured)?;
    let mut helper = remotes.launcher.launch(remote, &resolved)?;
    let caps = helper.capabilities()?;
    if let Some(name) = declared_by_the_helper_but_not_configured(&caps, &configured) {
        return Err(Refusal::MissingCredential {
            remote: remote.name.0.clone(),
            name: name.clone(),
        }
        .into());
    }
    accepted_kinds.set_accepted_kinds(&remote.name, &caps.kinds)?;
    Ok((helper, caps))
}

fn declared_by_the_helper_but_not_configured<'a>(
    caps: &'a RemoteCapabilities,
    configured: &[String],
) -> Option<&'a String> {
    caps.credentials
        .iter()
        .find(|name| !configured.contains(name))
}
