use dam_domain::ItemHash;

use super::FaultyStore;
use crate::ports::{FollowOn, FollowOnRepository, StoreError};

impl FollowOnRepository for FaultyStore {
    fn keep_follow_ons(&self, item: &ItemHash, follow_ons: &[FollowOn]) -> Result<(), StoreError> {
        self.write()?;
        self.inner.keep_follow_ons(item, follow_ons)
    }
    fn follow_ons(&self, item: &ItemHash) -> Result<Vec<FollowOn>, StoreError> {
        self.inner.follow_ons(item)
    }
    fn forget_follow_ons(&self, item: &ItemHash) -> Result<(), StoreError> {
        self.write()?;
        self.inner.forget_follow_ons(item)
    }
}
