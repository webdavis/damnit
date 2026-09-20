//! The remote-side and environment doubles every use-case test wires in.

use std::cell::RefCell;
use std::rc::Rc;

use dam_domain::{Date, Oid, Timestamp};
use dam_protocol::{Capabilities, Mutation, MutationResult, PullResponse, PushResponse};

use crate::config::{CredentialSpec, RemoteConfig};
use crate::ports::{
    Clock, CredentialError, CredentialSource, HelperError, HelperLauncher, Randomness, RemoteHelper,
};
use crate::secret::Secret;

pub fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

pub struct FixedRandom(pub u8);

impl Randomness for FixedRandom {
    fn fill(&mut self, buf: &mut [u8]) {
        buf.fill(self.0);
        self.0 = self.0.wrapping_add(1);
    }
}

pub struct FixedClock(pub Date);

impl Clock for FixedClock {
    fn today(&self) -> Date {
        self.0
    }
    fn now(&self) -> Timestamp {
        self.0
            .to_zoned(jiff::tz::TimeZone::UTC)
            .map(|z| z.timestamp())
            .unwrap_or(Timestamp::UNIX_EPOCH)
    }
}

type PushAnswer = Box<dyn Fn(&[Mutation]) -> Vec<MutationResult>>;
type LaunchedWith = Rc<RefCell<Vec<Vec<(String, Secret)>>>>;

/// A scripted helper: records what it was asked, answers what it was told.
pub struct ScriptedHelper {
    pub caps: Capabilities,
    pub pull_answer: PullResponse,
    pub push_answer: PushAnswer,
    pub pushed: Rc<RefCell<Vec<Mutation>>>,
    pub pulled_since: Rc<RefCell<Vec<Option<String>>>>,
}

impl std::fmt::Debug for ScriptedHelper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptedHelper").finish_non_exhaustive()
    }
}

impl RemoteHelper for ScriptedHelper {
    fn capabilities(&mut self) -> Result<Capabilities, HelperError> {
        Ok(self.caps.clone())
    }
    fn pull(&mut self, since: Option<&str>) -> Result<PullResponse, HelperError> {
        self.pulled_since
            .borrow_mut()
            .push(since.map(|s| s.to_string()));
        Ok(self.pull_answer.clone())
    }
    fn push(&mut self, mutations: Vec<Mutation>) -> Result<PushResponse, HelperError> {
        let results = (self.push_answer)(&mutations);
        self.pushed.borrow_mut().extend(mutations);
        Ok(PushResponse { results })
    }
}

pub struct ScriptedLauncher {
    pub make: Box<dyn Fn() -> ScriptedHelper>,
    pub launched_with: LaunchedWith,
}

impl HelperLauncher for ScriptedLauncher {
    fn launch(
        &self,
        _remote: &RemoteConfig,
        credentials: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        self.launched_with.borrow_mut().push(credentials.to_vec());
        Ok(Box::new((self.make)()))
    }
}

pub struct NoCredentials;
impl CredentialSource for NoCredentials {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError> {
        Ok(format!("value-of-{}", spec.name()).into())
    }
}

pub fn task_caps() -> Capabilities {
    Capabilities {
        protocol: 1,
        kinds: vec!["task".into()],
        fields: [
            "subject",
            "body",
            "path",
            "labels",
            "priority",
            "due",
            "deadline",
            "done",
            "recurrence",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
        credentials: vec!["api_token".into()],
        incremental: true,
    }
}
