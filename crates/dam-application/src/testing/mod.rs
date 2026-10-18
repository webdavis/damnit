pub mod contract;
mod doubles;
mod faulty_store;
mod memory_store;

pub use doubles::{
    EchoCredentials, FixedClock, FixedRandom, ScriptedHelper, ScriptedLauncher, oid, task_caps,
};
pub use faulty_store::FaultyStore;
pub use memory_store::MemoryStore;

pub mod prelude {
    pub use crate::ports::{
        AcceptedKindsRepository, CommitRepository, ConflictRepository, NoticeRepository,
        ObjectRepository, RemoteTrackingRepository, StageRepository,
    };
}
