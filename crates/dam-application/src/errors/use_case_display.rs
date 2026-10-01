use std::fmt;

use super::UseCaseError;
use crate::ports::{CredentialError, HelperError, StoreError};

impl fmt::Display for UseCaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UseCaseError::Refused(r) => write!(f, "{r}"),
            UseCaseError::Store(StoreError::Busy(why)) => {
                write!(f, "the store is busy, another dam is writing it: {why}")
            }
            UseCaseError::Store(StoreError::Failed(why)) => write!(f, "storage: {why}"),
            UseCaseError::Helper(HelperError::NotFound { helper }) => {
                write!(f, "{helper} was not found on PATH")
            }
            UseCaseError::Helper(HelperError::Protocol(s)) => {
                write!(f, "the helper answered something dam cannot read: {s}")
            }
            UseCaseError::Helper(HelperError::UnsupportedProtocol {
                helper,
                found,
                supported,
            }) => write!(
                f,
                "helper {helper} speaks protocol version {found}; dam speaks {supported}, so upgrade dam or use an older helper"
            ),
            UseCaseError::Helper(HelperError::CollidingCredentials {
                first,
                second,
                variable,
            }) => write!(
                f,
                "credentials {first} and {second} both become {variable}; rename one of them"
            ),
            UseCaseError::Helper(HelperError::Io(s)) => write!(f, "talking to the helper: {s}"),
            UseCaseError::Helper(HelperError::Remote(s)) => write!(f, "the remote refused: {s}"),
            UseCaseError::Helper(HelperError::Timeout {
                helper,
                deadline,
                said,
            }) => {
                write!(f, "helper {helper} gave no answer within {deadline}")?;
                match said {
                    Some(text) => write!(f, "; it said: {text}"),
                    None => Ok(()),
                }
            }
            UseCaseError::Helper(HelperError::Cancelled) => f.write_str("cancelled"),
            UseCaseError::Credential(CredentialError::Missing(n)) => {
                write!(f, "credential {n} has no source")
            }
            UseCaseError::Credential(CredentialError::CommandFailed { name, why }) => {
                write!(f, "the command for {name} failed: {why}")
            }
            UseCaseError::Credential(CredentialError::EnvUnset { name, var }) => {
                write!(f, "{name}: the variable {var} is not set")
            }
            UseCaseError::Editor(e) => write!(f, "editor: {}", e.0),
            UseCaseError::Parse(s) => write!(f, "{s}"),
        }
    }
}
