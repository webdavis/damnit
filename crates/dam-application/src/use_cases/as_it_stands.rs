use dam_domain::Object;

use crate::ports::{Declared, HelperError, RemoteHelper};
use crate::remote::{MutationOp, RemoteCapabilities, RemoteMutation};

pub(crate) type Wire<'a> = &'a dyn Fn(&RemoteMutation) -> Result<Declared, HelperError>;

pub(crate) fn as_it_stands(
    helper: &dyn RemoteHelper,
    caps: &RemoteCapabilities,
    object: &Object,
) -> Result<Declared, HelperError> {
    as_declared(&wire_of(helper), caps, object)
}

pub(crate) fn wire_of(
    helper: &dyn RemoteHelper,
) -> impl Fn(&RemoteMutation) -> Result<Declared, HelperError> + '_ {
    |m| helper.declared_on_the_wire(m)
}

pub(crate) fn as_declared(
    wire: Wire<'_>,
    caps: &RemoteCapabilities,
    object: &Object,
) -> Result<Declared, HelperError> {
    wire(&RemoteMutation {
        op: MutationOp::Update,
        oid: object.oid().clone(),
        idempotency_key: String::new(),
        remote_id: None,
        container_remote_id: None,
        container_oid: None,
        object: Some(object.clone()),
        fields: caps.fields.clone(),
    })
}
