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

pub(super) type Classified = (Vec<(Oid, Incoming)>, Vec<(Oid, String)>);

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
    let mut new_remote_id_mappings = Vec::new();

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
        object.base_mut().oid = oid.clone();
        let theirs_raw = object;
        if existing.is_none() {
            new_remote_id_mappings.push((oid.clone(), remote_id));
        }
        let local = repos.objects.get(&oid)?;
        if let Some((ours, theirs)) = local.as_ref().and_then(|l| kind_change(l, &theirs_raw)) {
            let notice = Notice::KindChanged {
                oid: oid.clone(),
                ours,
                theirs,
            };
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
                let last_seen_on_the_remote =
                    repos.remote_tracking.remote_snapshot(&remote.name, &oid)?;
                let ours_unmoved_since = last_seen_on_the_remote.as_ref() == Some(l);
                let theirs_unmoved_since = last_seen_on_the_remote.as_ref() == Some(&theirs);
                if ours_unmoved_since {
                    Incoming::FastForward(theirs)
                } else if theirs_unmoved_since {
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
    Ok((planned, new_remote_id_mappings))
}
