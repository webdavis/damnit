use dam_domain::{CommitId, Oid};

#[derive(Clone, Copy, Debug)]
pub enum Origin<'a> {
    Commit(&'a CommitId),
    Retry,
}

const COMMIT_BORNE_VERSION: char = '4';
const RETRY_BORNE_VERSION: char = '5';
const UUID_VARIANT: char = '8';
const ORDINAL_RESERVED_FOR_THE_HELPER: char = '0';

pub fn key(origin: Origin<'_>, oid: &Oid) -> String {
    let (version, source) = match origin {
        Origin::Retry => {
            let the_objects_own_tail = &oid.as_str()[25..];
            (RETRY_BORNE_VERSION, the_objects_own_tail)
        }
        Origin::Commit(id) => (COMMIT_BORNE_VERSION, &id.as_str()[..15]),
    };
    let object = oid.as_str();
    format!(
        "{}-{}-{version}{}-{UUID_VARIANT}{}-{}{ORDINAL_RESERVED_FOR_THE_HELPER}",
        &source[..8],
        &source[8..12],
        &source[12..15],
        &object[..3],
        &object[3..14],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oid(text: &str) -> Oid {
        Oid::parse(text).unwrap()
    }

    fn commit(text: &str) -> CommitId {
        CommitId::parse(text).unwrap()
    }

    const OBJECT: &str = "abcdef01234567890123456789abcdef01234567";
    const COMMIT: &str = "fedcba98765432100123456789abcdef01234567";

    #[test]
    fn a_key_is_uuid_shaped_with_the_last_character_reserved() {
        let k = key(Origin::Commit(&commit(COMMIT)), &oid(OBJECT));
        assert_eq!(k.len(), 36);
        let groups: Vec<&str> = k.split('-').collect();
        assert_eq!(
            groups.iter().map(|g| g.len()).collect::<Vec<_>>(),
            vec![8, 4, 4, 4, 12]
        );
        assert!(k.chars().all(|c| c == '-' || c.is_ascii_hexdigit()), "{k}");
        assert!(k.ends_with('0'), "the ordinal character is free: {k}");
    }

    #[test]
    fn the_same_change_yields_the_same_key_every_time() {
        let a = key(Origin::Commit(&commit(COMMIT)), &oid(OBJECT));
        let b = key(Origin::Commit(&commit(COMMIT)), &oid(OBJECT));
        assert_eq!(a, b);
    }

    #[test]
    fn a_different_commit_object_or_origin_yields_a_different_key() {
        let base = key(Origin::Commit(&commit(COMMIT)), &oid(OBJECT));
        let other_commit = key(
            Origin::Commit(&commit("0000000000000000000000000000000000000000")),
            &oid(OBJECT),
        );
        let other_object = key(
            Origin::Commit(&commit(COMMIT)),
            &oid("1111111111111111111111111111111111111111"),
        );
        let retry = key(Origin::Retry, &oid(OBJECT));
        assert_ne!(base, other_commit);
        assert_ne!(base, other_object);
        assert_ne!(base, retry);
        assert_ne!(
            retry,
            key(
                Origin::Retry,
                &oid("1111111111111111111111111111111111111111")
            )
        );
        assert_eq!(retry, key(Origin::Retry, &oid(OBJECT)));
    }
}
