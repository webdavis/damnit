//! The remote helper dam drives, and the credentials it is handed.

use crate::config::{CredentialSpec, RemoteConfig};
use crate::remote::{MutationOutcome, PullOutcome, RemoteCapabilities, RemoteMutation};
use crate::secret::Secret;

#[derive(Debug, PartialEq, Eq)]
pub enum HelperError {
    NotFound {
        helper: String,
    },
    Protocol(String),
    UnsupportedProtocol {
        helper: String,
        found: u32,
        supported: u32,
    },
    Io(String),
    Remote(String),
    Timeout {
        helper: String,
        deadline: String,
        /// The tail of what the helper wrote to its standard error, when it
        /// wrote anything.
        said: Option<String>,
    },
    Cancelled,
}

pub trait RemoteHelper: std::fmt::Debug {
    fn capabilities(&mut self) -> Result<RemoteCapabilities, HelperError>;
    fn pull(&mut self, since: Option<&str>) -> Result<PullOutcome, HelperError>;
    fn push(&mut self, mutations: Vec<RemoteMutation>)
    -> Result<Vec<MutationOutcome>, HelperError>;
}

pub trait HelperLauncher {
    fn launch(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum CredentialError {
    Missing(String),
    CommandFailed { name: String, why: String },
    EnvUnset { name: String, var: String },
}

pub trait CredentialSource {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError>;
}
