use dam_domain::Oid;

use crate::errors::UseCaseError;
use crate::ports::{Notice, RemoteName, Store};

pub(super) fn renew_failure_notices(
    store: &dyn Store,
    remote: &RemoteName,
    still_owed: &[Oid],
    failed: &[(Oid, String)],
) -> Result<(), UseCaseError> {
    for notice in store.notices()? {
        let Notice::PushFailed { remote: r, oid, .. } = &notice else {
            continue;
        };
        let failed_again = failed.iter().any(|(f, _)| f == oid);
        if r == remote && (failed_again || !still_owed.contains(oid)) {
            store.remove_notice(&notice)?;
        }
    }
    for (oid, why) in failed {
        store.add_notice(&Notice::PushFailed {
            remote: remote.clone(),
            oid: oid.clone(),
            why: why.clone(),
        })?;
    }
    Ok(())
}
