use std::collections::BTreeMap;

use dam_domain::{Field, Object, Oid, nearest_parents};

use crate::errors::UseCaseError;
use crate::ports::{RemoteName, RemoteTrackingRepository, Store};
use crate::remote::{MutationOp, RemoteMutation};
use crate::use_cases::push::owed::OwedChanges;

#[derive(Default)]
pub(super) struct Containers {
    unsettled: BTreeMap<Oid, Vec<Oid>>,
}

pub(super) enum Placement {
    Named(Option<String>),
    Held(String),
}

pub(super) fn places(mutation: &RemoteMutation) -> bool {
    mutation.op == MutationOp::Create || mutation.fields.contains(&Field::Path)
}

impl Containers {
    pub(super) fn new(
        store: &dyn Store,
        changes: &OwedChanges,
    ) -> Result<Containers, UseCaseError> {
        let unplaced: Vec<&Object> = changes
            .values()
            .filter_map(|change| change.after())
            .filter(|o| o.base().parent.is_none() && o.base().path.depth() > 0)
            .collect();
        let mut containers = Containers::default();
        if unplaced.is_empty() {
            return Ok(containers);
        }
        let mut committed = Vec::new();
        for object in store.all()? {
            committed.extend(store.committed(object.oid())?);
        }
        for object in unplaced {
            let could_hold = nearest_parents(&object.base().path, &committed);
            if could_hold.len() > 1 {
                containers
                    .unsettled
                    .insert(object.oid().clone(), could_hold);
            }
        }
        Ok(containers)
    }

    pub(super) fn place(
        &self,
        remote_tracking: &dyn RemoteTrackingRepository,
        remote: &RemoteName,
        object: &Object,
    ) -> Result<Placement, UseCaseError> {
        if let Some(could_hold) = self.unsettled.get(object.oid()) {
            return Ok(Placement::Held(not_known(object.oid(), could_hold)));
        }
        let Some(parent) = &object.base().parent else {
            return Ok(Placement::Named(None));
        };
        Ok(Placement::Named(remote_tracking.remote_id(remote, parent)?))
    }
}

fn not_known(oid: &Oid, could_hold: &[Oid]) -> String {
    let shorts: Vec<&str> = could_hold.iter().map(Oid::short).collect();
    let (last, rest) = shorts.split_last().expect("two or more could hold it");
    let (oid, first) = (oid.short(), shorts[0]);
    format!(
        "{oid}: dam does not know whether {} or {last} holds it; \
         run: dam pull, or name it: dam mv {oid} --parent {first}",
        rest.join(", ")
    )
}
