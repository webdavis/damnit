use super::*;

fn parsed_oid(text: &str) -> Oid {
    Oid::parse(text).unwrap()
}

fn commit(text: &str) -> CommitId {
    CommitId::parse(text).unwrap()
}

const OBJECT: &str = "abcdef01234567890123456789abcdef01234567";
const COMMIT: &str = "fedcba98765432100123456789abcdef01234567";

#[test]
fn a_key_is_uuid_shaped_with_the_last_character_reserved() {
    let k = key(Origin::Commit(&commit(COMMIT)), &parsed_oid(OBJECT));
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
    let a = key(Origin::Commit(&commit(COMMIT)), &parsed_oid(OBJECT));
    let b = key(Origin::Commit(&commit(COMMIT)), &parsed_oid(OBJECT));
    assert_eq!(a, b);
}

#[test]
fn a_different_commit_object_or_origin_yields_a_different_key() {
    let base = key(Origin::Commit(&commit(COMMIT)), &parsed_oid(OBJECT));
    let other_commit = key(
        Origin::Commit(&commit("0000000000000000000000000000000000000000")),
        &parsed_oid(OBJECT),
    );
    let other_object = key(
        Origin::Commit(&commit(COMMIT)),
        &parsed_oid("1111111111111111111111111111111111111111"),
    );
    let retry = key(Origin::Retry, &parsed_oid(OBJECT));
    assert_ne!(base, other_commit);
    assert_ne!(base, other_object);
    assert_ne!(base, retry);
    assert_ne!(
        retry,
        key(
            Origin::Retry,
            &parsed_oid("1111111111111111111111111111111111111111")
        )
    );
    assert_eq!(retry, key(Origin::Retry, &parsed_oid(OBJECT)));
}
