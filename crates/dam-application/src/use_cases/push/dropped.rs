use dam_domain::{CommitRecord, Object, Oid};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Attempts, DroppedBecause, Store};
use crate::remote::{MutationOp, RemoteCapabilities};
use crate::use_cases::push::mutations::goes_to;

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Dropped {
    pub(super) creates: Vec<(Object, DroppedBecause)>,
    pub(super) unused: Vec<Oid>,
}

impl Dropped {
    pub(super) fn absorb(&mut self, later: Dropped) {
        self.creates.extend(later.creates);
        self.unused.extend(later.unused);
    }
}

pub(super) fn creates_no_longer_sent(
    store: &dyn Store,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    unpushed: &[CommitRecord],
    record: &Attempts,
) -> Result<Dropped, UseCaseError> {
    let mut dropped = Dropped::default();
    for (oid, attempts) in record {
        if store.remote_id(&remote.name, oid)?.is_some() {
            continue;
        }
        let committed = store.committed(oid)?;
        let because = match &committed {
            None => DroppedBecause::Removed,
            Some(object) if goes_to(remote, caps, object) => continue,
            Some(_) => DroppedBecause::NoLongerSent,
        };
        let a_create = attempts.iter().any(|a| a.op == MutationOp::Create);
        let sent = last_sent(unpushed, oid, |o| goes_to(remote, caps, o)).or(committed);
        match sent.filter(|_| a_create) {
            Some(object) => dropped.creates.push((object, because)),
            None => dropped.unused.push(oid.clone()),
        }
    }
    Ok(dropped)
}

fn last_sent(
    unpushed: &[CommitRecord],
    oid: &Oid,
    goes_there: impl Fn(&Object) -> bool,
) -> Option<Object> {
    unpushed
        .iter()
        .rev()
        .flat_map(|record| record.changes.iter().rev())
        .map(|c| &c.change)
        .filter(|change| change.oid() == oid)
        .flat_map(|change| [change.after(), change.before()].into_iter().flatten())
        .find(|object| goes_there(object))
        .cloned()
}
