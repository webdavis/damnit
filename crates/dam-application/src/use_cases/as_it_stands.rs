use dam_domain::Object;

use crate::ports::{Declared, HelperError, RemoteHelper};
use crate::remote::{MutationOp, RemoteCapabilities, RemoteMutation};

pub(crate) fn as_it_stands(
    helper: &dyn RemoteHelper,
    caps: &RemoteCapabilities,
    object: &Object,
) -> Result<Declared, HelperError> {
    helper.declared_on_the_wire(&RemoteMutation {
        op: MutationOp::Update,
        oid: object.oid().clone(),
        idempotency_key: String::new(),
        remote_id: None,
        container_remote_id: None,
        object: Some(object.clone()),
        fields: caps.fields.clone(),
    })
}
