use crate::errors::{Refusal, UseCaseError};
use crate::ports::Transactional;

pub(super) fn rehearsed<T>(
    transaction: &dyn Transactional,
    work: impl FnOnce() -> Result<T, UseCaseError>,
) -> Result<T, UseCaseError> {
    let mut work = Some(work);
    let mut produced = None;
    let ran = transaction.in_transaction(&mut || {
        let run = work.take().expect("a rehearsal runs once");
        produced = Some(run()?);
        Err(Refusal::NeedsAnAnswer.into())
    });
    match produced {
        Some(produced) => Ok(produced),
        None => Err(ran.expect_err("a rehearsal never commits")),
    }
}
