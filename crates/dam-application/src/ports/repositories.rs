use super::store::{
    AcceptedKindsRepository, CommitRepository, ConflictRepository, NoticeRepository,
    ObjectRepository, RemoteTrackingRepository, StageRepository, Transactional,
};

pub trait Store:
    ObjectRepository
    + StageRepository
    + CommitRepository
    + RemoteTrackingRepository
    + AcceptedKindsRepository
    + ConflictRepository
    + NoticeRepository
    + Transactional
{
}

impl<T> Store for T where
    T: ObjectRepository
        + StageRepository
        + CommitRepository
        + RemoteTrackingRepository
        + AcceptedKindsRepository
        + ConflictRepository
        + NoticeRepository
        + Transactional
{
}

#[derive(Clone, Copy)]
pub struct Repositories<'a> {
    pub objects: &'a dyn ObjectRepository,
    pub stage: &'a dyn StageRepository,
    pub commits: &'a dyn CommitRepository,
    pub remote_tracking: &'a dyn RemoteTrackingRepository,
    pub accepted_kinds: &'a dyn AcceptedKindsRepository,
    pub conflicts: &'a dyn ConflictRepository,
    pub notices: &'a dyn NoticeRepository,
    pub transaction: &'a dyn Transactional,
}

impl<'a> Repositories<'a> {
    pub fn of(store: &'a dyn Store) -> Repositories<'a> {
        Repositories {
            objects: store,
            stage: store,
            commits: store,
            remote_tracking: store,
            accepted_kinds: store,
            conflicts: store,
            notices: store,
            transaction: store,
        }
    }
}
