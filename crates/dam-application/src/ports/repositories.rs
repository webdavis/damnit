use super::answers::AnswerRepository;
use super::attempts::AttemptRepository;
use super::days::DayRepository;
use super::follow_ons::FollowOnRepository;
use super::hook_marks::HookMarkRepository;
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
    + AttemptRepository
    + ConflictRepository
    + NoticeRepository
    + DayRepository
    + ItemRepository
    + FollowOnRepository
    + HookMarkRepository
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
        + AttemptRepository
        + ConflictRepository
        + NoticeRepository
        + DayRepository
        + ItemRepository
        + FollowOnRepository
        + HookMarkRepository
        + IdRepository
        + OriginRepository
        + Transactional
{
}
