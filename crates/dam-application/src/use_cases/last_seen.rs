use dam_domain::{Object, Oid};

use crate::ports::{RemoteName, Store, StoreError};

#[derive(Debug, PartialEq)]
pub(crate) enum LastSeen {
    Never,
    Held(Object),
    RemovedInTheAppNothingOwed(Object),
}

impl LastSeen {
    pub(crate) fn read(
        store: &dyn Store,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<LastSeen, StoreError> {
        let mapped = store.remote_id(remote, oid)?.is_some();
        Ok(match store.remote_snapshot(remote, oid)? {
            None => LastSeen::Never,
            Some(held) if mapped => LastSeen::Held(held),
            Some(kept) => LastSeen::RemovedInTheAppNothingOwed(kept),
        })
    }

    pub(crate) fn held(
        store: &dyn Store,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<Object>, StoreError> {
        Ok(match LastSeen::read(store, remote, oid)? {
            LastSeen::Held(held) => Some(held),
            LastSeen::Never | LastSeen::RemovedInTheAppNothingOwed(_) => None,
        })
    }

    pub(crate) fn baseline(
        store: &dyn Store,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<Object>, StoreError> {
        Ok(match LastSeen::read(store, remote, oid)? {
            LastSeen::Never => None,
            LastSeen::Held(o) | LastSeen::RemovedInTheAppNothingOwed(o) => Some(o),
        })
    }

    pub(crate) fn now_holds(
        store: &dyn Store,
        remote: &RemoteName,
        object: &Object,
    ) -> Result<(), StoreError> {
        store.set_remote_snapshot(remote, object)
    }

    pub(crate) fn adopted(
        store: &dyn Store,
        remote: &RemoteName,
        remote_id: &str,
        object: &Object,
    ) -> Result<(), StoreError> {
        store.map_remote_id(remote, object.oid(), remote_id)?;
        store.set_remote_snapshot(remote, object)
    }

    pub(crate) fn removed_in_the_app_nothing_owed(
        store: &dyn Store,
        remote: &RemoteName,
        kept: &Object,
    ) -> Result<(), StoreError> {
        store.clear_remote_mapping(remote, kept.oid())?;
        store.set_remote_snapshot(remote, kept)
    }

    pub(crate) fn gone(
        store: &dyn Store,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<(), StoreError> {
        store.clear_remote_mapping(remote, oid)
    }
}
