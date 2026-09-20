use crate::config::RemoteConfig;
use crate::credentials::resolve_credentials;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{CredentialSource, HelperLauncher, RemoteHelper};
use crate::remote::RemoteCapabilities;

/// Opens one helper process per remote operation, with the credentials the
/// remote's own config declares already in hand. The config names them, so
/// there is nothing to learn from the helper first.
pub(crate) fn connect(
    launcher: &dyn HelperLauncher,
    credentials: &dyn CredentialSource,
    remote: &RemoteConfig,
) -> Result<(Box<dyn RemoteHelper>, RemoteCapabilities), UseCaseError> {
    let configured: Vec<String> = remote
        .credentials
        .iter()
        .map(|spec| spec.name().to_string())
        .collect();
    let resolved = resolve_credentials(credentials, remote, &configured)?;
    let mut helper = launcher.launch(remote, &resolved)?;
    let caps = helper.capabilities()?;
    // The helper is the authority on what it needs, so a name it declares
    // that the config does not supply is still refused by name.
    if let Some(name) = caps
        .credentials
        .iter()
        .find(|name| !configured.iter().any(|c| &c == name))
    {
        return Err(Refusal::MissingCredential {
            remote: remote.name.0.clone(),
            name: name.clone(),
        }
        .into());
    }
    Ok((helper, caps))
}
