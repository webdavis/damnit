//! What one pulled object should do against the local state: create it, fast
//! forward it, raise a conflict, or nothing at all.

use dam_domain::{Object, Oid};

use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::merge::{kind_change, merge_fields};
use crate::ports::{Notice, Randomness, Repositories};
use crate::remote::{IncomingObject, RemoteCapabilities};

pub(super) enum Incoming {
    Create(Object),
    FastForward(Object),
    Conflict(Object),
    Unchanged,
}

/// What each pulled object should do, plus the oids generated for objects
/// with no local match yet, paired with the remote id each one maps to.
pub(super) type Classified = (Vec<(Oid, Incoming)>, Vec<(Oid, String)>);

/// Reads every pulled object against local state and decides what to do with
/// it. Notices are the only thing it writes; no object is touched. `fresh`
/// pairs a generated oid with the remote id it maps to, applied by the caller
/// once classification succeeds.
pub(super) fn classify(
    repos: Repositories<'_>,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    objects: Vec<IncomingObject>,
    random: &dyn Randomness,
) -> Result<Classified, UseCaseError> {
    let staged: Vec<Oid> = repos.stage.staged()?.into_iter().map(|c| c.oid).collect();
    let mut notices = repos.notices.notices()?;
    let mut planned = Vec::new();
    let mut fresh = Vec::new();

    for incoming in objects {
        let IncomingObject {
            remote_id,
            mut object,
        } = incoming;
        let existing = repos
            .remote_tracking
            .oid_for_remote_id(&remote.name, &remote_id)?;
        let oid = existing
            .clone()
            .unwrap_or_else(|| Oid::generate(&mut |b| random.fill(b)));
        // The remote knows the object by its own id; the oid it is tracked
        // by here is dam's to supply.
        object.base_mut().oid = oid.clone();
        let theirs_raw = object;
        if existing.is_none() {
            fresh.push((oid.clone(), remote_id));
        }
        let local = repos.objects.get(&oid)?;
        if let Some((ours, theirs)) = local.as_ref().and_then(|l| kind_change(l, &theirs_raw)) {
            let notice = Notice::KindChanged {
                oid: oid.clone(),
                ours,
                theirs,
            };
            // The kinds keep disagreeing until someone acts, so say it once.
            if !notices.contains(&notice) {
                repos.notices.add_notice(&notice)?;
                notices.push(notice);
            }
        }
        let theirs = merge_fields(
            local.as_ref().unwrap_or(&theirs_raw),
            &theirs_raw,
            &caps.fields,
        );
        let incoming = match local {
            None => Incoming::Create(theirs),
            Some(ref l) if *l == theirs => Incoming::Unchanged,
            Some(ref l) => {
                let base = repos.remote_tracking.remote_snapshot(&remote.name, &oid)?;
                if base.as_ref() == Some(l) {
                    Incoming::FastForward(theirs)
                } else if base.as_ref() == Some(&theirs) {
                    // The remote holds what it held when we last looked, so
                    // nothing is coming in; the difference is ours to push.
                    // This is also what stops a conflict already raised, and
                    // settled with ours, from being raised again on every pull
                    // until the resolution reaches the remote.
                    Incoming::Unchanged
                } else {
                    let committed = repos.objects.committed(&oid)?;
                    if committed.as_ref() != Some(l) || staged.contains(&oid) {
                        return Err(Refusal::DirtyOnPull { oid }.into());
                    }
                    Incoming::Conflict(theirs)
                }
            }
        };
        planned.push((oid, incoming));
    }
    Ok((planned, fresh))
}
