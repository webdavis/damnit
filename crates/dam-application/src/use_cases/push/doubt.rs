use std::collections::BTreeSet;

use crate::errors::UseCaseError;
use crate::ports::{RemoteName, Store};
use crate::use_cases::push::reconcile::Settlement;

pub(super) fn settle_doubt(
    store: &dyn Store,
    remote: &RemoteName,
    settlement: &Settlement,
) -> Result<(), UseCaseError> {
    let mut doubt = store.fields_in_doubt(remote)?;
    let mut touched = BTreeSet::new();
    for (oid, fields) in &settlement.doubted {
        doubt.entry(oid.clone()).or_default().extend(fields);
        touched.insert(oid);
    }
    for (oid, fields) in &settlement.confirmed {
        if let Some(doubted) = doubt.get_mut(oid) {
            doubted.retain(|f| !fields.contains(f));
        }
        touched.insert(oid);
    }
    for oid in &settlement.clear {
        doubt.remove(oid);
        touched.insert(oid);
    }
    for oid in touched {
        let left = doubt.get(oid).cloned().unwrap_or_default();
        store.set_fields_in_doubt(remote, oid, &left)?;
    }
    Ok(())
}
