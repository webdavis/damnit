use dam_domain::{Authored, ItemHash, Object};

use super::StoreError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectState {
    pub working: Object,
    pub owners_staged: Option<Authored>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FollowOn {
    pub before: ObjectState,
    pub after: ObjectState,
}

pub trait FollowOnRepository {
    fn keep_follow_ons(&self, item: &ItemHash, follow_ons: &[FollowOn]) -> Result<(), StoreError>;
    fn follow_ons(&self, item: &ItemHash) -> Result<Vec<FollowOn>, StoreError>;
    fn forget_follow_ons(&self, item: &ItemHash) -> Result<(), StoreError>;
}
