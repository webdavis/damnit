use std::collections::BTreeMap;

use dam_domain::{Change, Object, Oid, diff};

use crate::errors::UseCaseError;
use crate::ports::{
    CommitRepository, Conflict, Notice, ObjectRepository, RemoteName, Repositories, StageRepository,
};
use crate::use_cases::stage::tracked_oids;

pub struct Status {
    pub staged: Vec<Change>,
    pub unstaged: Vec<Change>,
    pub conflicts: Vec<Conflict>,
    pub notices: Vec<Notice>,
    pub unpushed: Vec<(RemoteName, usize)>,
}

pub fn status(repos: Repositories<'_>, remotes: &[RemoteName]) -> Result<Status, UseCaseError> {
    let mut unpushed = Vec::new();
    for remote in remotes {
        unpushed.push((remote.clone(), repos.commits.unpushed(remote)?.len()));
    }
    Ok(Status {
        staged: diff_staged(repos.stage)?,
        unstaged: diff_working(repos.objects, repos.stage, repos.commits)?,
        conflicts: repos.conflicts.conflicts()?,
        notices: repos.notices.notices()?,
        unpushed,
    })
}

pub fn diff_staged(stage: &dyn StageRepository) -> Result<Vec<Change>, UseCaseError> {
    Ok(stage.staged()?)
}

/// Working against what the stage would leave: committed with the stage applied.
/// Walks every tracked oid, not just working objects, so a working object
/// deleted but not yet staged still shows up as an unstaged delete.
pub fn diff_working(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    commits: &dyn CommitRepository,
) -> Result<Vec<Change>, UseCaseError> {
    let staged: BTreeMap<Oid, Change> = stage
        .staged()?
        .into_iter()
        .map(|c| (c.oid.clone(), c))
        .collect();
    let mut out = Vec::new();
    for oid in tracked_oids(objects, commits)? {
        let base: Option<Object> = match staged.get(&oid) {
            Some(change) => change.after.clone(),
            None => objects.committed(&oid)?,
        };
        let working = objects.get(&oid)?;
        if let Some(change) = diff(&oid, base.as_ref(), working.as_ref()) {
            out.push(change);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
    use crate::use_cases::commit::commit;
    use crate::use_cases::stage::add;
    use dam_domain::{Object, Op, Task};
    use jiff::civil::date;

    #[test]
    fn a_fully_staged_change_is_not_also_unstaged() {
        let store = MemoryStore::new();
        let repos = Repositories::of(&store);
        store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
        add(&store, &store, &[oid(1)]).unwrap();
        let s = status(repos, &[]).unwrap();
        assert_eq!(s.staged.len(), 1);
        assert!(s.unstaged.is_empty());
    }

    #[test]
    fn an_edit_after_staging_shows_as_unstaged_on_top_of_the_stage() {
        let store = MemoryStore::new();
        let repos = Repositories::of(&store);
        store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
        add(&store, &store, &[oid(1)]).unwrap();
        let mut t = store.get(&oid(1)).unwrap().unwrap();
        t.base_mut().subject = "b".into();
        store.put(&t).unwrap();
        let s = status(repos, &[]).unwrap();
        assert_eq!(s.staged[0].after.as_ref().unwrap().base().subject, "a");
        assert_eq!(s.unstaged[0].op, Op::Update);
        assert_eq!(s.unstaged[0].after.as_ref().unwrap().base().subject, "b");
    }

    #[test]
    fn unpushed_counts_per_remote() {
        let store = MemoryStore::new();
        let repos = Repositories::of(&store);
        store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
        add(&store, &store, &[oid(1)]).unwrap();
        commit(
            &store,
            &store,
            &FixedClock(date(2026, 9, 18)),
            &FixedRandom::new(5),
            "m",
        )
        .unwrap();
        let remote = RemoteName("todoist".into());
        let s = status(repos, std::slice::from_ref(&remote)).unwrap();
        assert_eq!(s.unpushed, vec![(remote, 1)]);
    }

    #[test]
    fn a_working_delete_not_yet_staged_shows_as_an_unstaged_delete() {
        let store = MemoryStore::new();
        store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
        add(&store, &store, &[oid(1)]).unwrap();
        commit(
            &store,
            &store,
            &FixedClock(date(2026, 9, 18)),
            &FixedRandom::new(5),
            "m",
        )
        .unwrap();
        store.delete(&oid(1)).unwrap();
        let s = diff_working(&store, &store, &store).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].op, Op::Delete);
        let staged = crate::use_cases::stage::add_all(&store, &store, &store).unwrap();
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].op, Op::Delete);
    }
}
