use dam_domain::{Kind, Oid};

use super::store::{RemoteName, StoreError};
use crate::remote::RejectedObject;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    RemovedUpstream {
        remote: RemoteName,
        oid: Oid,
        subject: String,
    },
    EventCancelled {
        oid: Oid,
        subject: String,
        attached: usize,
    },
    PushFailed {
        remote: RemoteName,
        oid: Oid,
        why: String,
    },
    PullFailed {
        remote: RemoteName,
        why: String,
    },
    PullSkipped {
        remote: RemoteName,
        object: RejectedObject,
    },
    KindChanged {
        oid: Oid,
        ours: Kind,
        theirs: Kind,
    },
}

pub trait NoticeRepository {
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError>;
    fn notices(&self) -> Result<Vec<Notice>, StoreError>;
    fn remove_notice(&self, notice: &Notice) -> Result<(), StoreError>;
    fn clear_notices(&self) -> Result<(), StoreError>;
}
