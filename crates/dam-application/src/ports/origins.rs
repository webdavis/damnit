use std::collections::BTreeMap;

use dam_domain::{Oid, Origin};

use super::StoreError;

pub trait OriginRepository {
    fn origins(&self) -> Result<BTreeMap<Oid, Origin>, StoreError>;
}
