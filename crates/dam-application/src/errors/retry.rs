use dam_domain::Oid;

#[derive(Debug, PartialEq, Eq)]
pub enum Retry {
    New(String),
    Move(Oid),
}
