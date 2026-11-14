use crate::config::{CredentialSpec, RemoteConfig};
use crate::ports::RemoteName;
use crate::remote::{PullOutcome, PushOutcome, RemoteCapabilities, RemoteMutation};
use crate::secret::Secret;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    CollidingCredentials {
        first: String,
        second: String,
        variable: String,
    },
    Remote(String),
    Timeout {
        helper: String,
        deadline: String,
        said: Option<String>,
    },
    Cancelled,
}

pub trait RemoteHelper: std::fmt::Debug {
    fn capabilities(&mut self) -> Result<RemoteCapabilities, HelperError>;
    fn pull(&mut self, since: Option<&str>) -> Result<PullOutcome, HelperError>;
    fn push(&mut self, mutations: Vec<RemoteMutation>) -> Result<PushOutcome, HelperError>;
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
    ComputeFailed {
        remote: RemoteName,
        name: String,
        at: String,
        why: ComputeFailure,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComputeFailure {
    Raised { message: Option<String> },
    ReturnedNonUtf8,
    ReturnedNotAString { returned: String },
}

pub trait CredentialSource {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError>;
}
