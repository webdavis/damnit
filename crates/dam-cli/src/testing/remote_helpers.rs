use std::cell::RefCell;

use dam_application::{
    CredentialError, CredentialSource, CredentialSpec, HelperError, HelperLauncher,
    MutationOutcome, MutationVerdict, PullOutcome, RemoteCapabilities, RemoteConfig, RemoteHelper,
    RemoteMutation, Secret,
};
use dam_domain::{Field, Kind};

pub(crate) struct MissingCredentials;
impl CredentialSource for MissingCredentials {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError> {
        Err(CredentialError::Missing(spec.name().to_string()))
    }
}

#[derive(Debug)]
pub(crate) struct EchoHelper {
    pub(crate) pulled: PullOutcome,
}
impl RemoteHelper for EchoHelper {
    fn capabilities(&mut self) -> Result<RemoteCapabilities, HelperError> {
        Ok(RemoteCapabilities {
            kinds: vec![Kind::Task],
            fields: vec![
                Field::Subject,
                Field::Body,
                Field::Path,
                Field::Labels,
                Field::Done,
                Field::Priority,
                Field::Due,
            ],
            credentials: vec![],
            incremental: false,
            removes_only_empty_containers: false,
        })
    }
    fn pull(&mut self, _: Option<&str>) -> Result<PullOutcome, HelperError> {
        Ok(std::mem::take(&mut self.pulled))
    }
    fn push(
        &mut self,
        mutations: Vec<RemoteMutation>,
    ) -> Result<Vec<MutationOutcome>, HelperError> {
        Ok(mutations
            .iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: MutationVerdict::Accepted {
                    remote_id: Some(format!("r-{}", &m.oid.as_str()[..4])),
                },
            })
            .collect())
    }
}

#[derive(Debug)]
pub(crate) struct EchoLauncher {
    pub(crate) pulled: RefCell<Option<PullOutcome>>,
}
impl HelperLauncher for EchoLauncher {
    fn launch(
        &self,
        _: &RemoteConfig,
        _: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        Ok(Box::new(EchoHelper {
            pulled: self.pulled.borrow_mut().take().unwrap_or_default(),
        }))
    }
}

#[derive(Debug)]
pub(crate) struct FailingLauncher;
impl HelperLauncher for FailingLauncher {
    fn launch(
        &self,
        remote: &RemoteConfig,
        _: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        Err(HelperError::NotFound {
            helper: remote.helper.clone(),
        })
    }
}
