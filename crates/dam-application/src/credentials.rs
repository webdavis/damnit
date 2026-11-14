use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{ComputeFailure, CredentialError, CredentialSource};
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

impl UseCaseError {
    pub(crate) fn withholding_what_credential_code_raised(self) -> UseCaseError {
        match self {
            UseCaseError::Credential(CredentialError::ComputeFailed {
                remote,
                name,
                at,
                why: ComputeFailure::Raised { .. },
            }) => UseCaseError::Credential(CredentialError::ComputeFailed {
                remote,
                name,
                at,
                why: ComputeFailure::Raised { message: None },
            }),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests;
