use super::notices::NoticeRepository;
use super::store::{
    AcceptedKindsRepository, CommitRepository, ConflictRepository, ObjectRepository,
    RemoteTrackingRepository, StageRepository, Transactional,
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
