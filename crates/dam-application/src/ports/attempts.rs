use std::collections::BTreeMap;

use dam_domain::{Field, Oid};

use super::{RemoteName, StoreError};
use crate::remote::MutationOp;

pub type Declared = BTreeMap<Field, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub key: String,
    pub op: MutationOp,
    pub container_remote_id: Option<String>,
    pub declared: Declared,
}

pub type Attempts = BTreeMap<Oid, Vec<Attempt>>;

pub trait AttemptRepository {
    fn attempts(&self, remote: &RemoteName) -> Result<Attempts, StoreError>;
    fn set_attempts(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        attempts: &[Attempt],
    ) -> Result<(), StoreError>;
}
