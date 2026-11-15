use dam_domain::{Object, Oid, kind_change, merge_fields};

use super::adopt::creates_found;
use super::ours::keep_what_we_sent;
use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
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

pub(super) type Classified = (Vec<(Oid, Incoming)>, Vec<(Oid, String)>);

pub(super) fn classify(
    store: &dyn Store,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    helper: &dyn RemoteHelper,
    objects: Vec<IncomingObject>,
    random: &dyn Randomness,
) -> Result<Classified, UseCaseError> {
    let staged: Vec<Oid> = store
        .staged()?
        .into_iter()
        .map(|c| c.change.oid().clone())
        .collect();
    let record = store.attempts(&remote.name)?;
    let adopted = creates_found(store, remote, helper, caps, &record, &objects)?;
    let mut notices = store.notices()?;
    let mut planned = Vec::new();
    let mut new_remote_id_mappings = Vec::new();

    for incoming in objects {
        let IncomingObject {
            remote_id,
            mut object,
        } = incoming;
        let existing = store.oid_for_remote_id(&remote.name, &remote_id)?;
        let adopting = adopted.get(&remote_id);
        let oid = existing
            .clone()
            .or_else(|| adopting.cloned())
            .unwrap_or_else(|| Oid::generate(&mut |b| random.fill(b)));
        object.base_mut().oid = oid.clone();
        let theirs_raw = object;
        let local = store.get(&oid)?;
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
        let theirs = merge_fields(
            local.as_ref().unwrap_or(&theirs_raw),
            &theirs_raw,
            &caps.fields,
        );
        match (&existing, adopting) {
            (Some(_), _) => {}
            (None, Some(_)) => LastSeen::adopted(store, &remote.name, &remote_id, &theirs)?,
            (None, None) => new_remote_id_mappings.push((oid.clone(), remote_id)),
        }
        let incoming = match local {
            None => Incoming::Create(theirs),
            Some(l) if l == theirs => settled(store, remote, theirs)?,
            Some(l) => {
                let attempts = record.get(&oid).map_or(&[][..], Vec::as_slice);
                let theirs = keep_what_we_sent(helper, caps, &l, theirs, attempts)?;
                against_local(store, remote, &staged, l, theirs)?
            }
        };
        planned.push((oid, incoming));
    }
    Ok((planned, new_remote_id_mappings))
}

fn settled(
    store: &dyn Store,
    remote: &RemoteConfig,
    theirs: Object,
) -> Result<Incoming, UseCaseError> {
    let last_seen_on_the_remote = LastSeen::held(store, &remote.name, theirs.oid())?;
    Ok(if last_seen_on_the_remote.as_ref() == Some(&theirs) {
        Incoming::Unchanged
    } else {
        Incoming::Settled(theirs)
    })
}

fn against_local(
    store: &dyn Store,
    remote: &RemoteConfig,
    staged: &[Oid],
    local: Object,
    theirs: Object,
) -> Result<Incoming, UseCaseError> {
    if local == theirs {
        return Ok(Incoming::Unchanged);
    }
    let oid = local.oid().clone();
    let last_seen_on_the_remote = LastSeen::held(store, &remote.name, &oid)?;
    if last_seen_on_the_remote.as_ref() == Some(&local) {
        return Ok(Incoming::FastForward(theirs));
    }
    if last_seen_on_the_remote.as_ref() == Some(&theirs) {
        return Ok(Incoming::Unchanged);
    }
    if store.committed(&oid)?.as_ref() != Some(&local) || staged.contains(&oid) {
        return Err(Refusal::DirtyOnPull { oid }.into());
    }
    Ok(Incoming::Conflict(Box::new(Conflict {
        remote: remote.name.clone(),
        oid,
        ours: local,
        theirs,
    })))
}
