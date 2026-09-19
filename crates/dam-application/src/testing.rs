use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use dam_domain::{Change, CommitId, CommitRecord, Date, Object, Oid, Path, Timestamp, coalesce};
use dam_protocol::{Capabilities, Mutation, MutationResult, PullResponse, PushResponse};

use crate::config::{CredentialSpec, RemoteConfig};
use crate::ports::{
    Clock, Conflict, CredentialError, CredentialSource, HelperError, HelperLauncher, Notice,
    ObjectStore, Randomness, RemoteHelper, RemoteName, StoreError,
};

pub(crate) fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

pub(crate) struct FixedRandom(pub u8);

impl Randomness for FixedRandom {
    fn fill(&mut self, buf: &mut [u8]) {
        buf.fill(self.0);
        self.0 = self.0.wrapping_add(1);
    }
}

pub(crate) struct FixedClock(pub Date);

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

#[derive(Default)]
struct Inner {
    working: BTreeMap<Oid, Object>,
    committed: BTreeMap<Oid, Object>,
    staged: BTreeMap<Oid, Change>,
    commits: Vec<CommitRecord>,
    pushed: BTreeMap<RemoteName, Vec<CommitId>>,
    remote_ids: BTreeMap<(RemoteName, Oid), String>,
    snapshots: BTreeMap<(RemoteName, Oid), Object>,
    sync: BTreeMap<RemoteName, String>,
    conflicts: BTreeMap<Oid, Conflict>,
    notices: Vec<Notice>,
    retries: BTreeMap<RemoteName, Vec<Oid>>,
}

#[derive(Default)]
pub(crate) struct MemoryStore(RefCell<Inner>);

impl MemoryStore {
    pub(crate) fn new() -> MemoryStore {
        MemoryStore::default()
    }
}

impl ObjectStore for MemoryStore {
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        Ok(self.0.borrow().working.get(oid).cloned())
    }
    fn all(&self) -> Result<Vec<Object>, StoreError> {
        Ok(self.0.borrow().working.values().cloned().collect())
    }
    fn children_of(&self, path: &Path) -> Result<Vec<Object>, StoreError> {
        Ok(self
            .0
            .borrow()
            .working
            .values()
            .filter(|o| o.base().path.parent().as_ref() == Some(path))
            .cloned()
            .collect())
    }
    fn dependents_of(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError> {
        Ok(self
            .0
            .borrow()
            .working
            .values()
            .filter(|o| o.base().depends.contains(oid))
            .map(|o| o.oid().clone())
            .collect())
    }
    fn put(&self, object: &Object) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .working
            .insert(object.oid().clone(), object.clone());
        Ok(())
    }
    fn delete(&self, oid: &Oid) -> Result<(), StoreError> {
        self.0.borrow_mut().working.remove(oid);
        Ok(())
    }
    fn committed(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        Ok(self.0.borrow().committed.get(oid).cloned())
    }
    fn stage(&self, change: Change) -> Result<(), StoreError> {
        let mut inner = self.0.borrow_mut();
        let oid = change.oid.clone();
        let merged = match inner.staged.remove(&oid) {
            Some(existing) => coalesce(existing, change),
            None => Some(change),
        };
        if let Some(c) = merged {
            inner.staged.insert(oid, c);
        }
        Ok(())
    }
    fn unstage(&self, oid: &Oid) -> Result<(), StoreError> {
        self.0.borrow_mut().staged.remove(oid);
        Ok(())
    }
    fn unstage_all(&self) -> Result<(), StoreError> {
        self.0.borrow_mut().staged.clear();
        Ok(())
    }
    fn staged(&self) -> Result<Vec<Change>, StoreError> {
        Ok(self.0.borrow().staged.values().cloned().collect())
    }
    fn commit(&self, record: &CommitRecord) -> Result<(), StoreError> {
        let mut inner = self.0.borrow_mut();
        for change in &record.changes {
            match &change.after {
                Some(after) => inner.committed.insert(change.oid.clone(), after.clone()),
                None => inner.committed.remove(&change.oid),
            };
            inner.staged.remove(&change.oid);
        }
        inner.commits.push(record.clone());
        Ok(())
    }
    fn log(&self) -> Result<Vec<CommitRecord>, StoreError> {
        Ok(self.0.borrow().commits.iter().rev().cloned().collect())
    }
    fn unpushed(&self, remote: &RemoteName) -> Result<Vec<CommitRecord>, StoreError> {
        let inner = self.0.borrow();
        let pushed = inner.pushed.get(remote).cloned().unwrap_or_default();
        Ok(inner
            .commits
            .iter()
            .filter(|c| !pushed.contains(&c.id))
            .cloned()
            .collect())
    }
    fn mark_pushed(&self, remote: &RemoteName, id: &CommitId) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .pushed
            .entry(remote.clone())
            .or_default()
            .push(id.clone());
        Ok(())
    }
    fn remote_id(&self, remote: &RemoteName, oid: &Oid) -> Result<Option<String>, StoreError> {
        Ok(self
            .0
            .borrow()
            .remote_ids
            .get(&(remote.clone(), oid.clone()))
            .cloned())
    }
    fn oid_for_remote_id(
        &self,
        remote: &RemoteName,
        remote_id: &str,
    ) -> Result<Option<Oid>, StoreError> {
        Ok(self
            .0
            .borrow()
            .remote_ids
            .iter()
            .find(|((r, _), id)| r == remote && id.as_str() == remote_id)
            .map(|((_, o), _)| o.clone()))
    }
    fn map_remote_id(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        remote_id: &str,
    ) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .remote_ids
            .insert((remote.clone(), oid.clone()), remote_id.to_string());
        Ok(())
    }
    fn remote_snapshot(
        &self,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<Object>, StoreError> {
        Ok(self
            .0
            .borrow()
            .snapshots
            .get(&(remote.clone(), oid.clone()))
            .cloned())
    }
    fn set_remote_snapshot(&self, remote: &RemoteName, object: &Object) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .snapshots
            .insert((remote.clone(), object.oid().clone()), object.clone());
        Ok(())
    }
    fn sync_token(&self, remote: &RemoteName) -> Result<Option<String>, StoreError> {
        Ok(self.0.borrow().sync.get(remote).cloned())
    }
    fn set_sync_token(&self, remote: &RemoteName, token: Option<&str>) -> Result<(), StoreError> {
        let mut inner = self.0.borrow_mut();
        match token {
            Some(t) => inner.sync.insert(remote.clone(), t.to_string()),
            None => inner.sync.remove(remote),
        };
        Ok(())
    }
    fn push_retries(&self, remote: &RemoteName) -> Result<Vec<Oid>, StoreError> {
        Ok(self
            .0
            .borrow()
            .retries
            .get(remote)
            .cloned()
            .unwrap_or_default())
    }
    fn set_push_retries(&self, remote: &RemoteName, oids: &[Oid]) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .retries
            .insert(remote.clone(), oids.to_vec());
        Ok(())
    }
    fn mark_conflict(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        theirs: &Object,
    ) -> Result<(), StoreError> {
        let ours = self
            .get(oid)?
            .ok_or_else(|| StoreError("conflict on a missing object".into()))?;
        self.0.borrow_mut().conflicts.insert(
            oid.clone(),
            Conflict {
                remote: remote.clone(),
                oid: oid.clone(),
                ours,
                theirs: theirs.clone(),
            },
        );
        Ok(())
    }
    fn conflicts(&self) -> Result<Vec<Conflict>, StoreError> {
        Ok(self.0.borrow().conflicts.values().cloned().collect())
    }
    fn clear_conflict(&self, oid: &Oid) -> Result<(), StoreError> {
        self.0.borrow_mut().conflicts.remove(oid);
        Ok(())
    }
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError> {
        self.0.borrow_mut().notices.push(notice.clone());
        Ok(())
    }
    fn notices(&self) -> Result<Vec<Notice>, StoreError> {
        Ok(self.0.borrow().notices.clone())
    }
    fn clear_notices(&self) -> Result<(), StoreError> {
        self.0.borrow_mut().notices.clear();
        Ok(())
    }
}

type PushAnswer = Box<dyn Fn(&[Mutation]) -> Vec<MutationResult>>;
type LaunchedWith = Rc<RefCell<Vec<Vec<(String, String)>>>>;

/// A scripted helper: records what it was asked, answers what it was told.
pub(crate) struct ScriptedHelper {
    pub caps: Capabilities,
    pub pull_answer: PullResponse,
    pub push_answer: PushAnswer,
    pub pushed: Rc<RefCell<Vec<Mutation>>>,
    pub pulled_since: Rc<RefCell<Vec<Option<String>>>>,
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

pub(crate) struct ScriptedLauncher {
    pub make: Box<dyn Fn() -> ScriptedHelper>,
    pub launched_with: LaunchedWith,
}

impl HelperLauncher for ScriptedLauncher {
    fn launch(
        &self,
        _remote: &RemoteConfig,
        credentials: &[(String, String)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        self.launched_with.borrow_mut().push(credentials.to_vec());
        Ok(Box::new((self.make)()))
    }
}

pub(crate) struct NoCredentials;
impl CredentialSource for NoCredentials {
    fn resolve(&self, spec: &CredentialSpec) -> Result<String, CredentialError> {
        Ok(format!("value-of-{}", spec.name()))
    }
}

pub(crate) fn task_caps() -> Capabilities {
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
