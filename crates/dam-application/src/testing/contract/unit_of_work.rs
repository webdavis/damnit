use crate::errors::UseCaseError;
use crate::ports::{Store, StoreError};
use dam_domain::Change;

use super::{oid, task};

pub fn transactional_contract(store: &dyn Store) {
    store
        .in_transaction(&mut || {
            store.put(&task(1, "kept", ""))?;
            store.put_staged(Change::Create {
                after: task(1, "kept", ""),
            })?;
            Ok(())
        })
        .expect("work that succeeds commits");
    assert!(store.get(&oid(1)).unwrap().is_some());
    assert_eq!(store.staged().unwrap().len(), 1);

    let failed = store.in_transaction(&mut || {
        store.put(&task(2, "discarded", ""))?;
        store.unstage_all()?;
        Err(UseCaseError::Store(StoreError::Failed("stopped".into())))
    });
    assert_eq!(
        failed,
        Err(UseCaseError::Store(StoreError::Failed("stopped".into()))),
        "the reason the work stopped reaches the caller unchanged"
    );
    assert_eq!(
        store.get(&oid(2)).unwrap(),
        None,
        "a write made before the failure is rolled back"
    );
    assert_eq!(
        store.staged().unwrap().len(),
        1,
        "and so is a write to another record family in the same unit"
    );
}
