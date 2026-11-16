use std::collections::BTreeMap;

use dam_domain::{Field, Object, Oid, kind_change, merge_fields};

use super::compare::{against_local, changed_in_the_app, settled};

use super::adopt::creates_found;
use super::ours::keep_what_we_sent;
use super::parents::{Parents, RemotePaths, as_seen};
use super::tally::Tally;
use super::untouched::{AtThePull, Untouched};
use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Conflict, Notice, Randomness, RemoteHelper, Store};
use crate::remote::{IncomingObject, RemoteCapabilities};
use crate::use_cases::last_seen::LastSeen;

pub(super) enum Incoming {
    Create(Object),
    FastForward(Object),
    Conflict(Box<Conflict>),
    Settled(Object),
    Unchanged,
}

pub(super) type Classified = (Vec<(Oid, Incoming)>, Vec<(Oid, String)>, RemotePaths);

pub(super) fn classify(
    store: &dyn Store,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    helper: &dyn RemoteHelper,
    (objects, container_remote_ids): (Vec<IncomingObject>, BTreeMap<String, String>),
    random: &dyn Randomness,
    tally: &mut Tally,
) -> Result<Classified, UseCaseError> {
    let staged: Vec<Oid> = store
        .staged()?
        .into_iter()
        .map(|c| c.change.oid().clone())
        .collect();
    let record = store.attempts(&remote.name)?;
    let adopted = creates_found(store, remote, helper, caps, &record, &objects)?;
    tally.creates_matched.extend(adopted.values().cloned());
    let mut notices = store.notices()?;
    let mut planned = Vec::new();
    let mut new_remote_id_mappings = Vec::new();
    let mut remote_paths = RemotePaths::new();
    let mut oids = Vec::new();
    for incoming in &objects {
        let existing = store.oid_for_remote_id(&remote.name, &incoming.remote_id)?;
        oids.push(
            existing
                .or_else(|| adopted.get(&incoming.remote_id).cloned())
                .unwrap_or_else(|| Oid::generate(&mut |b| random.fill(b))),
        );
    }
    let parents = Parents::new(container_remote_ids, &objects, &oids);

    for (incoming, oid) in objects.into_iter().zip(oids) {
        let IncomingObject {
            remote_id,
            mut object,
        } = incoming;
        let existing = store.oid_for_remote_id(&remote.name, &remote_id)?;
        let adopting = adopted.get(&remote_id);
        object.base_mut().oid = oid.clone();
        let rebuilt = parents.name(store, remote, &remote_id, &mut object)?;
        let theirs_raw = object;
        let local = store.get(&oid)?;
        let committed = store.committed(&oid)?;
        if let Some((ours, theirs)) = local.as_ref().and_then(|l| kind_change(l, &theirs_raw)) {
            let notice = Notice::KindChanged {
                oid: oid.clone(),
                ours,
                theirs,
            };
            if !notices.contains(&notice) {
                store.add_notice(&notice)?;
                notices.push(notice);
            }
        }
        let here = local.as_ref().or(committed.as_ref()).unwrap_or(&theirs_raw);
        let mut theirs = merge_fields(here, &theirs_raw, &caps.fields);
        if let Some(path) = rebuilt
            && caps.fields.contains(&Field::Path)
        {
            remote_paths.insert(oid.clone(), theirs_raw.base().path.clone());
            theirs.base_mut().path = path;
        }
        let seen = as_seen(&theirs, &remote_paths);
        match (&existing, adopting) {
            (Some(_), _) => {}
            (None, Some(_)) => LastSeen::adopted(store, &remote.name, &remote_id, &seen)?,
            (None, None) => new_remote_id_mappings.push((oid.clone(), remote_id)),
        }
        let facts = AtThePull {
            uncommitted: local != committed || staged.contains(&oid),
            changed_in_the_app: changed_in_the_app(store, remote, caps, &theirs_raw)?,
            the_pull_would_write: local.as_ref() != Some(&theirs),
        };
        let incoming = match (Untouched::of(facts), local) {
            (Some(Untouched::LeftAlone), _) => Incoming::Unchanged,
            (Some(Untouched::Conflict), ours) => Incoming::Conflict(Box::new(Conflict {
                remote: remote.name.clone(),
                oid: oid.clone(),
                ours,
                theirs,
            })),
            (None, None) => Incoming::Create(theirs),
            (None, Some(l)) if l == theirs => settled(store, remote, theirs, &remote_paths)?,
            (None, Some(l)) => {
                let attempts = record.get(&oid).map_or(&[][..], Vec::as_slice);
                let (theirs, kept) = keep_what_we_sent(helper, caps, &l, theirs, attempts)?;
                tally.values_kept.push((oid.clone(), kept));
                against_local(store, remote, l, theirs, &remote_paths)?
            }
        };
        planned.push((oid, incoming));
    }
    Ok((planned, new_remote_id_mappings, remote_paths))
}
