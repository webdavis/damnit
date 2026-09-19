use dam_domain::{Change, CommitId, CommitRecord, EventStatus, Object, Oid, Op};
use dam_protocol::Capabilities;

use crate::config::{Config, RemoteConfig};
use crate::errors::{Refusal, UseCaseError};
use crate::merge::merge_fields;
use crate::ports::{
    Clock, CredentialSource, HelperLauncher, Notice, ObjectStore, Randomness, RemoteName,
};
use crate::use_cases::connect::connect;
use crate::use_cases::push::select_remotes;
use crate::wire::from_wire;

#[derive(Debug)]
pub struct PullReport {
    pub remote: RemoteName,
    pub created: usize,
    pub updated: usize,
    pub conflicts: usize,
    pub removed_upstream: usize,
    pub unchanged: usize,
}

enum Incoming {
    Create(Object),
    FastForward(Object),
    Conflict(Object),
    Unchanged,
}

pub fn pull(
    store: &dyn ObjectStore,
    launcher: &dyn HelperLauncher,
    credentials: &dyn CredentialSource,
    clock: &dyn Clock,
    random: &mut dyn Randomness,
    config: &Config,
    remote: Option<&str>,
) -> Result<Vec<PullReport>, UseCaseError> {
    let mut reports = Vec::new();
    for target in select_remotes(config, remote)? {
        let (mut helper, caps) = connect(launcher, credentials, target)?;
        let since = store.sync_token(&target.name)?;
        let response = helper.pull(since.as_deref())?;
        reports.push(apply(store, clock, random, target, &caps, response)?);
    }
    Ok(reports)
}

fn apply(
    store: &dyn ObjectStore,
    clock: &dyn Clock,
    random: &mut dyn Randomness,
    remote: &RemoteConfig,
    caps: &Capabilities,
    response: dam_protocol::PullResponse,
) -> Result<PullReport, UseCaseError> {
    let staged: Vec<Oid> = store.staged()?.into_iter().map(|c| c.oid).collect();
    let mut planned: Vec<(Oid, Incoming)> = Vec::new();
    let mut fresh = Vec::new();

    for mut wire in response.objects {
        let remote_id = wire
            .remote_id
            .clone()
            .ok_or_else(|| UseCaseError::Parse("a pulled object has no remote_id".into()))?;
        let oid = match store.oid_for_remote_id(&remote.name, &remote_id)? {
            Some(o) => o,
            None => {
                let o = Oid::generate(&mut |b| random.fill(b));
                fresh.push((o.clone(), remote_id));
                o
            }
        };
        wire.oid = oid.to_string();
        let theirs_raw = from_wire(&wire).map_err(UseCaseError::Parse)?;
        let local = store.get(&oid)?;
        let theirs = merge_fields(
            local.as_ref().unwrap_or(&theirs_raw),
            &theirs_raw,
            &caps.fields,
        );
        let incoming = match local {
            None => Incoming::Create(theirs),
            Some(ref l) if *l == theirs => Incoming::Unchanged,
            Some(ref l) => {
                let base = store.remote_snapshot(&remote.name, &oid)?;
                if base.as_ref() == Some(l) {
                    Incoming::FastForward(theirs)
                } else {
                    let committed = store.committed(&oid)?;
                    if committed.as_ref() != Some(l) || staged.contains(&oid) {
                        return Err(Refusal::DirtyOnPull { oid }.into());
                    }
                    Incoming::Conflict(theirs)
                }
            }
        };
        planned.push((oid, incoming));
    }

    let mut report = PullReport {
        remote: remote.name.clone(),
        created: 0,
        updated: 0,
        conflicts: 0,
        removed_upstream: 0,
        unchanged: 0,
    };
    for (oid, remote_id) in fresh {
        store.map_remote_id(&remote.name, &oid, &remote_id)?;
    }
    let mut landed = Vec::new();
    for (oid, incoming) in planned {
        match incoming {
            Incoming::Unchanged => report.unchanged += 1,
            Incoming::Create(o) | Incoming::FastForward(o) => {
                let is_create = store.get(&oid)?.is_none();
                store.put(&o)?;
                store.set_remote_snapshot(&remote.name, &o)?;
                landed.push((
                    oid,
                    if is_create { Op::Create } else { Op::Update },
                    o.clone(),
                ));
                if is_create {
                    report.created += 1
                } else {
                    report.updated += 1
                }
                note_cancelled(store, &o)?;
            }
            Incoming::Conflict(o) => {
                store.mark_conflict(&remote.name, &oid, &o)?;
                store.set_remote_snapshot(&remote.name, &o)?;
                report.conflicts += 1;
            }
        }
    }
    if !landed.is_empty() {
        let record = CommitRecord {
            id: CommitId::generate(&mut |b| random.fill(b)),
            message: format!("pull from {}", remote.name.0),
            at: clock.now(),
            changes: landed
                .into_iter()
                .map(|(oid, op, after)| Change {
                    oid,
                    op,
                    before: None,
                    after: Some(after),
                })
                .collect(),
        };
        store.commit(&record)?;
        store.mark_pushed(&remote.name, &record.id)?;
    }
    for remote_id in response.removed {
        if let Some(oid) = store.oid_for_remote_id(&remote.name, &remote_id)? {
            let subject = store
                .get(&oid)?
                .map(|o| o.base().subject.clone())
                .unwrap_or_default();
            store.add_notice(&Notice::RemovedUpstream {
                remote: remote.name.clone(),
                oid,
                subject,
            })?;
            report.removed_upstream += 1;
        }
    }
    store.set_sync_token(&remote.name, response.sync.as_deref())?;
    Ok(report)
}

fn note_cancelled(store: &dyn ObjectStore, object: &Object) -> Result<(), UseCaseError> {
    if let Object::Event(e) = object
        && e.status == EventStatus::Cancelled
    {
        let attached = store
            .all()?
            .iter()
            .filter(|o| {
                o.as_task()
                    .is_some_and(|t| t.event.as_ref() == Some(&e.base.oid))
            })
            .count();
        if attached > 0 {
            store.add_notice(&Notice::EventCancelled {
                oid: e.base.oid.clone(),
                subject: e.base.subject.clone(),
                attached,
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
