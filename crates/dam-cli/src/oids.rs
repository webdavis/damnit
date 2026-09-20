use dam_application::{ObjectRepository, Refusal, UseCaseError};
use dam_domain::Oid;

use crate::error::CliError;

const MIN_PREFIX: usize = 4;

/// Resolves the oid argument every verb that takes one accepts: a full oid, or a
/// hex prefix at least `MIN_PREFIX` characters long that matches exactly one object.
pub(crate) fn resolve_oid(objects: &dyn ObjectRepository, text: &str) -> Result<Oid, CliError> {
    if let Ok(oid) = Oid::parse(text) {
        return Ok(oid);
    }
    if text.len() < MIN_PREFIX || !text.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(CliError::Usage(format!(
            "{text:?} is not an oid; give at least {MIN_PREFIX} hex characters"
        )));
    }
    let matches: Vec<Oid> = objects
        .all()?
        .into_iter()
        .map(|o| o.oid().clone())
        .filter(|o| o.as_str().starts_with(text))
        .collect();
    match matches.as_slice() {
        [] => Err(UseCaseError::Refused(Refusal::NoSuchObject(text.to_string())).into()),
        [one] => Ok(one.clone()),
        _ => Err(CliError::Ambiguous {
            text: text.to_string(),
            matches,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_adapters::SqliteStore;
    use dam_domain::{Object, Task};

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    #[test]
    fn a_full_oid_and_a_unique_prefix_resolve() {
        let store = SqliteStore::in_memory().unwrap();
        store.put(&Object::Task(Task::new(oid(0xab), "a"))).unwrap();
        store.put(&Object::Task(Task::new(oid(0xac), "b"))).unwrap();
        assert_eq!(resolve_oid(&store, oid(0xab).as_str()).unwrap(), oid(0xab));
        assert_eq!(resolve_oid(&store, "abab").unwrap(), oid(0xab));
    }

    #[test]
    fn short_ambiguous_and_unknown_prefixes_are_refused() {
        let store = SqliteStore::in_memory().unwrap();
        let twin = |last: u8| {
            Oid::generate(&mut |x: &mut [u8]| {
                x.fill(0xab);
                x[19] = last;
            })
        };
        store.put(&Object::Task(Task::new(twin(1), "a"))).unwrap();
        store.put(&Object::Task(Task::new(twin(2), "b"))).unwrap();
        assert!(matches!(resolve_oid(&store, "ab"), Err(CliError::Usage(_))));
        assert!(matches!(
            resolve_oid(&store, "zzzz"),
            Err(CliError::Usage(_))
        ));
        assert!(matches!(
            resolve_oid(&store, "cdcd"),
            Err(CliError::UseCase(_))
        ));
        assert!(matches!(
            resolve_oid(&store, "abab"),
            Err(CliError::Ambiguous { .. })
        ));
    }
}
