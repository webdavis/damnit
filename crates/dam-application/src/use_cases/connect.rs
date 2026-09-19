use dam_protocol::Capabilities;

use crate::config::RemoteConfig;
use crate::credentials::resolve_credentials;
use crate::errors::UseCaseError;
use crate::ports::{CredentialSource, HelperLauncher, RemoteHelper};

/// Reads capabilities with no credentials, then relaunches with the ones the
/// helper declared. A helper that declares none is used as launched.
pub(crate) fn connect(
    launcher: &dyn HelperLauncher,
    credentials: &dyn CredentialSource,
    remote: &RemoteConfig,
) -> Result<(Box<dyn RemoteHelper>, Capabilities), UseCaseError> {
    let mut probe = launcher.launch(remote, &[])?;
    let caps = probe.capabilities()?;
    if caps.credentials.is_empty() {
        return Ok((probe, caps));
    }
    let resolved = resolve_credentials(credentials, remote, &caps.credentials)?;
    let helper = launcher.launch(remote, &resolved)?;
    Ok((helper, caps))
}
