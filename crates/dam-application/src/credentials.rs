use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::CredentialSource;
use crate::secret::Secret;

pub fn resolve_credentials(
    source: &dyn CredentialSource,
    remote: &RemoteConfig,
    declared: &[String],
) -> Result<Vec<(String, Secret)>, UseCaseError> {
    let mut out = Vec::with_capacity(declared.len());
    for name in declared {
        let spec = remote
            .credentials
            .iter()
            .find(|s| s.name() == name)
            .ok_or_else(|| Refusal::MissingCredential {
                remote: remote.name.0.clone(),
                name: name.clone(),
            })?;
        out.push((name.clone(), source.resolve(spec)?));
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
