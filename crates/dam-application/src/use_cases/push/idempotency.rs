use dam_domain::{CommitId, Oid};

/// What a mutation is a resend of: the commit whose change it carries, or the
/// retry set, which holds an oid alone once its commit has been accounted for.
#[derive(Clone, Copy, Debug)]
pub enum Origin<'a> {
    Commit(&'a CommitId),
    Retry,
}

/// The key a mutation carries, and carries again on every resend, so a remote
/// that deduplicates by key does the work once however often an interrupted
/// push repeats it.
///
/// It is UUID-shaped because that is the shape a Todoist sync command's `uuid`
/// takes. The characters are dam's own identifiers rather than a digest of
/// them: a commit id names the change and an oid names the object, each already
/// forty random hex characters, so disjoint slices of the pair are unique
/// without any further mixing. The version character separates a commit-borne
/// key from a retry-borne one, and the last character is reserved, so a helper
/// that turns one mutation into several remote commands can number them there
/// without minting anything of its own.
pub fn key(origin: Origin<'_>, oid: &Oid) -> String {
    let (version, source) = match origin {
        // A retry has no commit to name, so the object's own tail stands in.
        Origin::Retry => ('5', &oid.as_str()[25..]),
        Origin::Commit(id) => ('4', &id.as_str()[..15]),
    };
    let object = oid.as_str();
    format!(
        "{}-{}-{version}{}-8{}-{}0",
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

    /// The whole point: the same change resent carries the same key.
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
