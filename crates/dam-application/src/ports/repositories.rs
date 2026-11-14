use super::answers::AnswerRepository;
use super::days::DayRepository;
use super::ids::IdRepository;
use super::items::ItemRepository;
use super::notices::NoticeRepository;
use super::origins::OriginRepository;
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
    + AnswerRepository
    + ConflictRepository
    + NoticeRepository
    + DayRepository
    + ItemRepository
    + IdRepository
    + OriginRepository
    + Transactional
{
}

impl<T> Store for T where
    T: ObjectRepository
        + StageRepository
        + CommitRepository
        + RemoteTrackingRepository
        + AcceptedKindsRepository
        + AnswerRepository
        + ConflictRepository
        + NoticeRepository
        + DayRepository
        + ItemRepository
        + IdRepository
        + OriginRepository
        + Transactional
{
}
