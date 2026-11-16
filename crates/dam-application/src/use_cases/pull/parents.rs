use std::collections::BTreeMap;

use dam_domain::{Object, Oid, Path};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::Store;
use crate::remote::IncomingObject;
use crate::use_cases::last_seen::LastSeen;

pub(super) type RemotePaths = BTreeMap<Oid, Path>;

pub(super) fn as_seen(theirs: &Object, remote_paths: &RemotePaths) -> Object {
    let mut seen = theirs.clone();
    if let Some(path) = remote_paths.get(theirs.oid()) {
        seen.base_mut().path = path.clone();
    }
    seen
}

pub(super) struct Parents {
    named: BTreeMap<String, String>,
    pulled: BTreeMap<String, (Oid, Object)>,
}

impl Parents {
    pub(super) fn new(
        named: BTreeMap<String, String>,
        objects: &[IncomingObject],
        oids: &[Oid],
    ) -> Parents {
        let pulled = objects
            .iter()
            .zip(oids)
            .map(|(o, oid)| (o.remote_id.clone(), (oid.clone(), o.object.clone())))
            .collect();
        Parents { named, pulled }
    }

    pub(super) fn name(
        &self,
        store: &dyn Store,
        remote: &RemoteConfig,
        remote_id: &str,
        theirs: &mut Object,
    ) -> Result<Option<Path>, UseCaseError> {
        let Some(container) = self.named.get(remote_id) else {
            return Ok(None);
        };
        let (parent, as_pulled) = match self.pulled.get(container) {
            Some((oid, object)) => (oid.clone(), Some(object.clone())),
            None => match store.oid_for_remote_id(&remote.name, container)? {
                Some(oid) if store.get(&oid)?.is_some() => {
                    let seen = LastSeen::baseline(store, &remote.name, &oid)?;
                    (oid, seen)
                }
                _ => return Ok(None),
            },
        };
        let here = store.get(&parent)?.or_else(|| as_pulled.clone());
        let folder = |o: Option<Object>| o.and_then(|o| o.base().owned_folder());
        theirs.base_mut().parent = Some(parent);
        settle(store, remote, theirs)?;
        let path = &theirs.base().path;
        Ok(Some(match (folder(as_pulled), folder(here)) {
            (Some(from), Some(to)) => path.rebase(&from, &to).unwrap_or(path.clone()),
            _ => path.clone(),
        }))
    }
}

fn settle(store: &dyn Store, remote: &RemoteConfig, theirs: &Object) -> Result<(), UseCaseError> {
    let (oid, parent, path) = (theirs.oid(), &theirs.base().parent, &theirs.base().path);
    let unsettled_at_the_pulled_path =
        |o: &Object| o.base().parent.is_none() && &o.base().path == path;
    if let Some(mut seen) = LastSeen::held(store, &remote.name, oid)?
        && unsettled_at_the_pulled_path(&seen)
    {
        seen.base_mut().parent = parent.clone();
        LastSeen::now_holds(store, &remote.name, &seen)?;
    }
    if let (Some(committed), Some(parent)) = (store.committed(oid)?, parent)
        && unsettled_at_the_pulled_path(&committed)
    {
        store.settle_committed_parent(oid, parent)?;
    }
    if let Some(mut working) = store.get(oid)?
        && unsettled_at_the_pulled_path(&working)
    {
        working.base_mut().parent = parent.clone();
        store.put(&working)?;
    }
    Ok(())
}
