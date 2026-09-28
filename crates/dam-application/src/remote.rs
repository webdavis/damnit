use dam_domain::{Field, Kind, Object, Oid};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteCapabilities {
    pub kinds: Vec<Kind>,
    pub fields: Vec<Field>,
    pub credentials: Vec<String>,
    pub incremental: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationOp {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteMutation {
    pub op: MutationOp,
    pub oid: Oid,
    pub idempotency_key: String,
    pub remote_id: Option<String>,
    pub object: Option<Object>,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutationOutcome {
    pub oid: Oid,
    pub ok: bool,
    pub remote_id: Option<String>,
    pub why: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingObject {
    pub remote_id: String,
    pub object: Object,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedObject {
    pub remote_id: String,
    pub why: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PullOutcome {
    pub objects: Vec<IncomingObject>,
    pub rejected: Vec<RejectedObject>,
    pub removed: Vec<String>,
    pub cancelled: Vec<String>,
    pub sync: Option<String>,
}
