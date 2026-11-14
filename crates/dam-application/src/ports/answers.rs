use dam_domain::Oid;

use super::{RemoteName, StoreError};

pub trait AnswerRepository {
    fn answers(&self, remote: &RemoteName, oid: &Oid) -> Result<u64, StoreError>;
    fn count_answer(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError>;
}
