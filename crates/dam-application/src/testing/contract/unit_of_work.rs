//! One unit of work across several record families.

use crate::errors::UseCaseError;
use crate::ports::{ObjectRepository, StageRepository, StoreError, Transactional};
use dam_domain::{Change, Op};

use super::{oid, task};

/// A unit of work lands whole or not at all, across record families, and a
/// failure comes back to the caller unchanged.
pub fn transactional_contract(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    transaction: &dyn Transactional,
) {
    transaction
        .in_transaction(&mut || {
            objects.put(&task(1, "kept", ""))?;
            stage.stage(Change {
                oid: oid(1),
                op: Op::Create,
                before: None,
                after: Some(task(1, "kept", "")),
            })?;
            Ok(())
        })
        .expect("work that succeeds commits");
    assert!(objects.get(&oid(1)).unwrap().is_some());
    assert_eq!(stage.staged().unwrap().len(), 1);

    let failed = transaction.in_transaction(&mut || {
        objects.put(&task(2, "discarded", ""))?;
        stage.unstage_all()?;
        Err(UseCaseError::Store(StoreError::Failed("stopped".into())))
    });
    assert_eq!(
        failed,
        Err(UseCaseError::Store(StoreError::Failed("stopped".into()))),
        "the reason the work stopped reaches the caller unchanged"
    );
    assert_eq!(
        objects.get(&oid(2)).unwrap(),
        None,
        "a write made before the failure is rolled back"
    );
    assert_eq!(
        stage.staged().unwrap().len(),
        1,
        "and so is a write to another record family in the same unit"
    );
}
