use dam_domain::{Object, Oid, kind_change, merge_fields};

use super::PullReport;
use super::adopt::creates_found;
use super::ours::keep_what_we_sent;
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

pub(super) type Classified = (Vec<(Oid, Incoming)>, Vec<(Oid, String)>);

pub(super) fn classify(
    store: &dyn Store,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    helper: &dyn RemoteHelper,
    objects: Vec<IncomingObject>,
    random: &dyn Randomness,
    report: &mut PullReport,
) -> Result<Classified, UseCaseError> {
    let staged: Vec<Oid> = store
        .staged()?
        .into_iter()
        .map(|c| c.change.oid().clone())
        .collect();
    let record = store.attempts(&remote.name)?;
    let adopted = creates_found(store, remote, helper, caps, &record, &objects)?;
    report.creates_matched += adopted.len();
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
        let theirs = merge_fields(here, &theirs_raw, &caps.fields);
        match (&existing, adopting) {
            (Some(_), _) => {}
            (None, Some(_)) => LastSeen::adopted(store, &remote.name, &remote_id, &theirs)?,
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
            (None, Some(l)) if l == theirs => settled(store, remote, theirs)?,
            (None, Some(l)) => {
                let attempts = record.get(&oid).map_or(&[][..], Vec::as_slice);
                let (theirs, kept) = keep_what_we_sent(helper, caps, &l, theirs, attempts)?;
                report.values_kept += kept;
                against_local(store, remote, l, theirs)?
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
    Ok(Incoming::Conflict(Box::new(Conflict {
        remote: remote.name.clone(),
        oid,
        ours: Some(local),
        theirs,
    })))
}

fn changed_in_the_app(
    store: &dyn Store,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    theirs: &Object,
) -> Result<bool, UseCaseError> {
    let last_seen = LastSeen::held(store, &remote.name, theirs.oid())?;
    Ok(last_seen.is_none_or(|seen| merge_fields(&seen, theirs, &caps.fields) != seen))
}
