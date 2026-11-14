use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Change, Field, Object, Oid};

use crate::errors::UseCaseError;
use crate::ports::{RemoteName, RemoteTrackingRepository, Store};
use crate::remote::RemoteCapabilities;
use crate::use_cases::push::owed::OwedChanges;

#[derive(Default)]
pub(super) struct Refiles(BTreeMap<Oid, Refile>);

struct Refile {
    object: Object,
    owner: Oid,
}

pub(super) struct Refiled {
    pub(super) container_remote_id: Option<String>,
}

impl Refiles {
    pub(super) fn new(
        store: &dyn Store,
        remote: &RemoteName,
        caps: &RemoteCapabilities,
        changes: &OwedChanges,
    ) -> Result<Refiles, UseCaseError> {
        let mut refiles = Refiles::default();
        if !caps.fields.contains(&Field::Path) {
            return Ok(refiles);
        }
        let mut folders_losing_their_owner = Vec::new();
        for (oid, change) in changes {
            if let Change::Delete { .. } = change
                && let Some(held) = store.remote_snapshot(remote, oid)?
                && let Some(folder) = held.base().owned_folder()
            {
                folders_losing_their_owner.push(folder);
            }
        }
        if folders_losing_their_owner.is_empty() {
            return Ok(refiles);
        }
        let local = store.all()?.into_iter().map(|o| o.oid().clone());
        let known: BTreeSet<Oid> = local.chain(changes.keys().cloned()).collect();
        let mut after_the_push = Vec::new();
        for oid in known {
            let Some(after) = store.committed(&oid)? else {
                continue;
            };
            let upstream = store.remote_snapshot(remote, &oid)?;
            if upstream.is_some() || changes.contains_key(&oid) {
                after_the_push.push((after, upstream.map(|held| held.base().path.clone())));
            }
        }
        for folder in folders_losing_their_owner {
            let mut owners = after_the_push
                .iter()
                .filter(|(after, _)| after.base().owned_folder().as_ref() == Some(&folder));
            let (Some((owner, _)), None) = (owners.next(), owners.next()) else {
                continue;
            };
            let staying_in_the_folder = after_the_push.iter().filter(|(after, held_at)| {
                held_at.as_ref() == Some(&folder) && after.base().path == folder
            });
            for (after, _) in staying_in_the_folder {
                let refile = Refile {
                    object: after.clone(),
                    owner: owner.oid().clone(),
                };
                refiles.0.insert(after.oid().clone(), refile);
            }
        }
        Ok(refiles)
    }

    pub(super) fn refiled(
        &self,
        remote_tracking: &dyn RemoteTrackingRepository,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<Refiled>, UseCaseError> {
        let Some(refile) = self.0.get(oid) else {
            return Ok(None);
        };
        let container_remote_id = remote_tracking.remote_id(remote, &refile.owner)?;
        Ok(Some(Refiled {
            container_remote_id,
        }))
    }

    pub(super) fn owe(&self, changes: &mut OwedChanges) {
        for (oid, refile) in &self.0 {
            changes
                .entry(oid.clone())
                .or_insert_with(|| Change::Update {
                    before: refile.object.clone(),
                    after: refile.object.clone(),
                });
        }
    }
}
