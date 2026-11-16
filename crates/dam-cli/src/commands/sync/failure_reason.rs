use dam_application::{CredentialError, UseCaseError};

use crate::error::CliError;

pub(super) fn failure_reason(failed: &CliError) -> String {
    match failed {
        CliError::UseCase(UseCaseError::Credential(CredentialError::ComputeFailed {
            remote,
            name,
            ..
        })) => format!(
            "computing {name} failed; run dam sync run {} to see why",
            remote.0
        ),
        failed => failed
            .to_string()
            .lines()
            .next()
            .unwrap_or_default()
            .to_string(),
    }
}
