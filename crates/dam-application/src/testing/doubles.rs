use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dam_domain::{Date, Field, Kind, Oid, Timestamp};

use crate::config::{CredentialSpec, RemoteConfig};
use crate::ports::{
    Clock, CredentialError, CredentialSource, HelperError, HelperLauncher, Randomness, RemoteHelper,
};
use crate::remote::{PullOutcome, PushOutcome, RemoteCapabilities, RemoteMutation};
use crate::secret::Secret;

pub fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

pub struct FixedRandom {
    byte_for_the_next_fill: Cell<u8>,
}

impl FixedRandom {
    pub fn new(byte_for_the_first_fill: u8) -> FixedRandom {
        FixedRandom {
            byte_for_the_next_fill: Cell::new(byte_for_the_first_fill),
        }
    }
}

impl Randomness for FixedRandom {
    fn fill(&self, buf: &mut [u8]) {
        let byte = self.byte_for_the_next_fill.get();
        buf.fill(byte);
        self.byte_for_the_next_fill.set(byte.wrapping_add(1));
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
    fn zone(&self) -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::UTC
    }
}

type PushAnswer = Box<dyn Fn(&[RemoteMutation]) -> Result<PushOutcome, HelperError>>;
type LaunchedWith = Rc<RefCell<Vec<Vec<(String, Secret)>>>>;

pub struct ScriptedHelper {
    pub caps: RemoteCapabilities,
    pub pull_answer: Result<PullOutcome, HelperError>,
    pub push_answer: PushAnswer,
    pub pushed: Rc<RefCell<Vec<RemoteMutation>>>,
    pub pulled_since: Rc<RefCell<Vec<Option<String>>>>,
}

impl std::fmt::Debug for ScriptedHelper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptedHelper").finish_non_exhaustive()
    }
}

impl RemoteHelper for ScriptedHelper {
    fn capabilities(&mut self) -> Result<RemoteCapabilities, HelperError> {
        Ok(self.caps.clone())
    }
    fn pull(&mut self, since: Option<&str>) -> Result<PullOutcome, HelperError> {
        self.pulled_since
            .borrow_mut()
            .push(since.map(|s| s.to_string()));
        self.pull_answer.clone()
    }
    fn push(&mut self, mutations: Vec<RemoteMutation>) -> Result<PushOutcome, HelperError> {
        let answer = (self.push_answer)(&mutations)?;
        self.pushed.borrow_mut().extend(mutations);
        Ok(answer)
    }
    fn declared_on_the_wire(&self, mutation: &RemoteMutation) -> String {
        let values = mutation
            .object
            .as_ref()
            .map(|o| dam_domain::carrying_only(o, &mutation.fields));
        format!("{:?} {values:?}", mutation.fields)
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

pub struct EchoCredentials;
impl CredentialSource for EchoCredentials {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError> {
        Ok(match spec {
            CredentialSpec::Literal { value, .. } => value.clone(),
            CredentialSpec::Computed { remote, name } => format!("{}.{name}", remote.0).into(),
        })
    }
}

pub fn task_caps() -> RemoteCapabilities {
    RemoteCapabilities {
        kinds: vec![Kind::Task],
        fields: vec![
            Field::Subject,
            Field::Body,
            Field::Path,
            Field::Labels,
            Field::Priority,
            Field::Due,
            Field::Deadline,
            Field::Done,
            Field::Recurrence,
        ],
        credentials: vec!["api_token".into()],
        incremental: true,
        removes_only_empty_containers: false,
    }
}
