use std::collections::BTreeMap;

use dam_domain::{Change, Object, Oid, diff};

use crate::errors::UseCaseError;
use crate::ports::{Conflict, Notice, ObjectStore, RemoteName};

pub struct Status {
    pub staged: Vec<Change>,
    pub unstaged: Vec<Change>,
    pub conflicts: Vec<Conflict>,
    pub notices: Vec<Notice>,
    pub unpushed: Vec<(RemoteName, usize)>,
}

pub fn status(store: &dyn ObjectStore, remotes: &[RemoteName]) -> Result<Status, UseCaseError> {
    let mut unpushed = Vec::new();
    for remote in remotes {
        unpushed.push((remote.clone(), store.unpushed(remote)?.len()));
    }
    Ok(Status {
        staged: diff_staged(store)?,
        unstaged: diff_working(store)?,
        conflicts: store.conflicts()?,
        notices: store.notices()?,
        unpushed,
    })
}

pub fn diff_staged(store: &dyn ObjectStore) -> Result<Vec<Change>, UseCaseError> {
    Ok(store.staged()?)
}

/// Working against what the stage would leave: committed with the stage applied.
pub fn diff_working(store: &dyn ObjectStore) -> Result<Vec<Change>, UseCaseError> {
    let staged: BTreeMap<Oid, Change> = store
        .staged()?
        .into_iter()
        .map(|c| (c.oid.clone(), c))
        .collect();
    let mut out = Vec::new();
    for object in store.all()? {
        let oid = object.oid().clone();
        let base: Option<Object> = match staged.get(&oid) {
            Some(change) => change.after.clone(),
            None => store.committed(&oid)?,
        };
        if let Some(change) = diff(&oid, base.as_ref(), Some(&object)) {
            out.push(change);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
    use crate::use_cases::commit::commit;
    use crate::use_cases::stage::add;
    use dam_domain::{Object, Op, Task};
    use jiff::civil::date;

    #[test]
    fn a_fully_staged_change_is_not_also_unstaged() {
        let store = MemoryStore::new();
        store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
        add(&store, &[oid(1)]).unwrap();
        let s = status(&store, &[]).unwrap();
        assert_eq!(s.staged.len(), 1);
        assert!(s.unstaged.is_empty());
    }

    #[test]
    fn an_edit_after_staging_shows_as_unstaged_on_top_of_the_stage() {
        let store = MemoryStore::new();
        store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
        add(&store, &[oid(1)]).unwrap();
        let mut t = store.get(&oid(1)).unwrap().unwrap();
        t.base_mut().subject = "b".into();
        store.put(&t).unwrap();
        let s = status(&store, &[]).unwrap();
        assert_eq!(s.staged[0].after.as_ref().unwrap().base().subject, "a");
        assert_eq!(s.unstaged[0].op, Op::Update);
        assert_eq!(s.unstaged[0].after.as_ref().unwrap().base().subject, "b");
    }

    #[test]
    fn unpushed_counts_per_remote() {
        let store = MemoryStore::new();
        store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
        add(&store, &[oid(1)]).unwrap();
        commit(
            &store,
            &FixedClock(date(2026, 9, 18)),
            &mut FixedRandom(5),
            "m",
        )
        .unwrap();
        let remote = RemoteName("todoist".into());
        let s = status(&store, std::slice::from_ref(&remote)).unwrap();
        assert_eq!(s.unpushed, vec![(remote, 1)]);
    }
}
