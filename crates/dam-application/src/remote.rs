//! What dam asks a remote for and what it gets back, in dam's own terms.
//! The bytes a helper reads and writes are the adapter's business.

use dam_domain::{Field, Kind, Object, Oid};

/// What a remote accepts. A remote never receives a field it did not declare,
/// and a pulled object never overwrites a field it did not declare.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteCapabilities {
    pub kinds: Vec<Kind>,
    pub fields: Vec<Field>,
    pub credentials: Vec<String>,
    pub incremental: bool,
}

/// What one mutation asks the remote to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationOp {
    Create,
    Update,
    Delete,
}

/// One change dam asks a remote to make.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteMutation {
    pub op: MutationOp,
    pub oid: Oid,
    /// Stable across every resend of this same mutation, so a remote that
    /// deduplicates by key does the work once however often an interrupted
    /// push repeats it.
    pub idempotency_key: String,
    pub remote_id: Option<String>,
    /// Absent for a delete, which names the object by its remote id alone.
    pub object: Option<Object>,
    pub fields: Vec<Field>,
}

/// What the remote did with one mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutationOutcome {
    pub oid: Oid,
    pub ok: bool,
    pub remote_id: Option<String>,
    pub why: Option<String>,
}

/// One object as a remote holds it. The remote does not know the oid dam
/// tracks it by, so `object` carries a placeholder identity and the caller
/// sets the oid it resolved from `remote_id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingObject {
    pub remote_id: String,
    pub object: Object,
}

/// An object the remote sent that dam could not read. It is reported and
/// skipped; the rest of the pull lands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedObject {
    pub remote_id: String,
    pub why: String,
}

/// Everything one pull brought back.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PullOutcome {
    pub objects: Vec<IncomingObject>,
    pub rejected: Vec<RejectedObject>,
    /// Remote ids the remote no longer has.
    pub removed: Vec<String>,
    /// Remote ids the remote cancelled: each moves the event it maps to cancelled, through the same rules as any pulled change.
    pub cancelled: Vec<String>,
    /// Where the remote's incremental sync now stands.
    pub sync: Option<String>,
}
