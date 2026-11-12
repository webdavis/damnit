use dam_domain::{CallerId, IdHolder, IdKey, Oid};

use super::StoreError;

pub trait IdRepository {
    fn oid_for_id(&self, key: &IdKey) -> Result<Option<Oid>, StoreError>;
    fn id_for_oid(&self, holder: &IdHolder, oid: &Oid) -> Result<Option<CallerId>, StoreError>;
    fn map_id(&self, key: &IdKey, oid: &Oid) -> Result<(), StoreError>;
}
