use dam_domain::{Change, Oid};

use super::PullReport;
use super::classify::Incoming;
use super::notices::note_cancelled;
use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Clock, Randomness, Repositories};
use crate::use_cases::commit::record;

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
                let committed = repos.objects.committed(&oid)?;
                landed.extend(Change::between(committed, Some(o.clone())));
                if is_create {
                    report.created += 1
                } else {
                    report.updated += 1
                }
                note_cancelled(repos.objects, repos.notices, &o)?;
            }
            Incoming::Conflict(conflict) => {
                repos.conflicts.mark_conflict(&conflict)?;
                repos
                    .remote_tracking
                    .set_remote_snapshot(&remote.name, &conflict.theirs)?;
                report.conflicts += 1;
                note_cancelled(repos.objects, repos.notices, &conflict.theirs)?;
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
    landed: Vec<Change>,
) -> Result<(), UseCaseError> {
    let pulled = record(
        clock,
        random,
        format!("pull from {}", remote.name.0),
        landed,
    );
    repos.commits.commit(&pulled)?;
    repos.commits.mark_pushed(&remote.name, &pulled.id)?;
    Ok(())
}
