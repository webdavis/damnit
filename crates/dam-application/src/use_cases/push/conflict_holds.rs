use std::collections::BTreeSet;

use dam_domain::{Change, Object, Oid, Path};

use crate::errors::UseCaseError;
use crate::ports::Store;
use crate::use_cases::push::owed::OwedChanges;

pub(super) struct Owed {
    pub(super) changes: OwedChanges,
    pub(super) held: BTreeSet<Oid>,
}

impl From<OwedChanges> for Owed {
    fn from(changes: OwedChanges) -> Owed {
        Owed {
            changes,
            held: BTreeSet::new(),
        }
    }
}

#[derive(Default)]
struct Held {
    oids: BTreeSet<Oid>,
    places: Vec<Path>,
    folders: Vec<Path>,
}

impl Held {
    fn hold(&mut self, oid: Oid, objects: [Option<&Object>; 2]) {
        self.oids.insert(oid);
        for object in objects.into_iter().flatten() {
            self.places.push(object.base().path.clone());
            self.folders.extend(object.base().owned_folder());
        }
    }

    fn waits_on_it(&self, change: &Change) -> bool {
        match change {
            Change::Delete { before } => before
                .base()
                .owned_folder()
                .is_some_and(|folder| self.places.iter().any(|place| place.is_within(&folder))),
            _ => {
                let landing = &change.object().base().path;
                let placed = change.before().is_none_or(|b| &b.base().path != landing);
                placed && self.folders.iter().any(|folder| landing.is_within(folder))
            }
        }
    }
}

pub(super) fn hold_for_conflicts(
    store: &dyn Store,
    changes: OwedChanges,
) -> Result<Owed, UseCaseError> {
    let mut held = Held::default();
    for conflict in store.conflicts()? {
        let owed = changes.get(&conflict.oid);
        let upstream = [conflict.ours.as_ref(), Some(&conflict.theirs)];
        held.hold(conflict.oid.clone(), upstream);
        held.hold(
            conflict.oid,
            [owed.and_then(Change::before), owed.and_then(Change::after)],
        );
    }
    loop {
        let waiting: Vec<Oid> = changes
            .iter()
            .filter(|(oid, change)| !held.oids.contains(*oid) && held.waits_on_it(change))
            .map(|(oid, _)| oid.clone())
            .collect();
        if waiting.is_empty() {
            break;
        }
        for oid in waiting {
            let change = &changes[&oid];
            held.hold(oid.clone(), [change.before(), change.after()]);
        }
    }
    let (withheld, sent) = changes
        .into_iter()
        .partition::<OwedChanges, _>(|(oid, _)| held.oids.contains(oid));
    Ok(Owed {
        changes: sent,
        held: withheld.into_keys().collect(),
    })
}
