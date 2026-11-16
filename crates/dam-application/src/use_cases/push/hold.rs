use dam_domain::Oid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hold {
    pub oid: Oid,
    pub why: String,
    pub waits_on: Vec<Oid>,
}

pub(super) fn reasons(holds: Vec<Hold>) -> impl Iterator<Item = (Oid, String)> {
    holds.into_iter().map(|hold| (hold.oid, hold.why))
}
