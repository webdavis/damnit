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
mod tests;
