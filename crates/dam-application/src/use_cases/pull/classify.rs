use dam_domain::{Object, Oid, kind_change, merge_fields};

use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Conflict, Notice, Randomness, Store};
use crate::remote::{IncomingObject, RemoteCapabilities};

pub(super) enum Incoming {
    Create(Object),
    FastForward(Object),
    Conflict(Box<Conflict>),
    Unchanged,
}

pub(super) type Classified = (Vec<(Oid, Incoming)>, Vec<(Oid, String)>);

pub(super) fn classify(
    store: &dyn Store,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    objects: Vec<IncomingObject>,
    random: &dyn Randomness,
) -> Result<Classified, UseCaseError> {
    let staged: Vec<Oid> = store
        .staged()?
        .into_iter()
        .map(|c| c.oid().clone())
        .collect();
    let mut notices = store.notices()?;
    let mut planned = Vec::new();
    let mut new_remote_id_mappings = Vec::new();

    for incoming in objects {
        let IncomingObject {
            remote_id,
            mut object,
        } = incoming;
        let existing = store.oid_for_remote_id(&remote.name, &remote_id)?;
        let oid = existing
            .clone()
            .unwrap_or_else(|| Oid::generate(&mut |b| random.fill(b)));
        object.base_mut().oid = oid.clone();
        let theirs_raw = object;
        if existing.is_none() {
            new_remote_id_mappings.push((oid.clone(), remote_id));
        }
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
        let incoming = match local {
            None => Incoming::Create(theirs),
            Some(ref l) if *l == theirs => Incoming::Unchanged,
            Some(ref l) => {
                let last_seen_on_the_remote = store.remote_snapshot(&remote.name, &oid)?;
                let ours_unmoved_since = last_seen_on_the_remote.as_ref() == Some(l);
                let theirs_unmoved_since = last_seen_on_the_remote.as_ref() == Some(&theirs);
                if ours_unmoved_since {
                    Incoming::FastForward(theirs)
                } else if theirs_unmoved_since {
                    Incoming::Unchanged
                } else {
                    let committed = store.committed(&oid)?;
                    if committed.as_ref() != Some(l) || staged.contains(&oid) {
                        return Err(Refusal::DirtyOnPull { oid }.into());
                    }
                    Incoming::Conflict(Box::new(Conflict {
                        remote: remote.name.clone(),
                        oid: oid.clone(),
                        ours: l.clone(),
                        theirs,
                    }))
                }
            }
        };
        planned.push((oid, incoming));
    }
    Ok((planned, new_remote_id_mappings))
}
