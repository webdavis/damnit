use crate::config::{CredentialSpec, RemoteConfig};
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{ComputeFailure, CredentialError, CredentialSource, Progress, Step};
use crate::secret::Secret;

pub fn resolve_credentials(
    source: &dyn CredentialSource,
    remote: &RemoteConfig,
    declared: &[String],
    progress: &dyn Progress,
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
        out.push((name.clone(), resolved(source, spec, progress)?));
    }
    Ok(out)
}

fn resolved(
    source: &dyn CredentialSource,
    spec: &CredentialSpec,
    progress: &dyn Progress,
) -> Result<Secret, CredentialError> {
    let CredentialSpec::Computed { remote, name } = spec else {
        return source.resolve(spec);
    };
    progress.start(Step {
        doing: &format!("reading {} {name} from secrets.lua", remote.0),
        did: &format!("read {} {name} from secrets.lua", remote.0),
        unit: "",
        total: None,
    });
    let secret = source.resolve(spec);
    match &secret {
        Ok(_) => progress.finish(),
        Err(_) => progress.fail(),
    }
    secret
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
