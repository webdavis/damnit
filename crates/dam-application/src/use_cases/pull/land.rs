use dam_domain::{Change, CommitId, CommitRecord, Object, Oid, Op};

use super::PullReport;
use super::classify::Incoming;
use super::notices::note_cancelled;
use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Clock, Randomness, Repositories};

pub(super) fn land(
    repos: Repositories<'_>,
    clock: &dyn Clock,
    random: &dyn Randomness,
    remote: &RemoteConfig,
    planned: Vec<(Oid, Incoming)>,
    report: &mut PullReport,
) -> Result<(), UseCaseError> {
    let mut landed = Vec::new();
    for (oid, incoming) in planned {
        match incoming {
            Incoming::Unchanged => report.unchanged += 1,
            Incoming::Create(o) | Incoming::FastForward(o) => {
                let is_create = repos.objects.get(&oid)?.is_none();
                repos.objects.put(&o)?;
                repos
                    .remote_tracking
                    .set_remote_snapshot(&remote.name, &o)?;
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
                note_cancelled(repos.objects, repos.notices, &o)?;
            }
            Incoming::Conflict(o) => {
                repos.conflicts.mark_conflict(&remote.name, &oid, &o)?;
                repos
                    .remote_tracking
                    .set_remote_snapshot(&remote.name, &o)?;
                report.conflicts += 1;
                note_cancelled(repos.objects, repos.notices, &o)?;
            }
        }
    }
    if landed.is_empty() {
        return Ok(());
    }
    commit_as_already_pushed(repos, clock, random, remote, landed)
}

fn commit_as_already_pushed(
    repos: Repositories<'_>,
    clock: &dyn Clock,
    random: &dyn Randomness,
    remote: &RemoteConfig,
    landed: Vec<(Oid, Op, Object)>,
) -> Result<(), UseCaseError> {
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
    repos.commits.commit(&record)?;
    repos.commits.mark_pushed(&remote.name, &record.id)?;
    Ok(())
}
