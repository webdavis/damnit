use crate::errors::UseCaseError;
use crate::ports::Transactional;

pub(crate) fn atomically<T>(
    transaction: &dyn Transactional,
    work: impl FnOnce() -> Result<T, UseCaseError>,
) -> Result<T, UseCaseError> {
    let mut work = Some(work);
    let mut produced = None;
    transaction.in_transaction(&mut || {
        let run = work.take().expect("a unit of work runs once");
        produced = Some(run()?);
        Ok(())
    })?;
    Ok(produced.expect("a committed unit of work produced its value"))
}

#[cfg(test)]
mod tests;
