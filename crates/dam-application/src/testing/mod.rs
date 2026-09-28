pub mod contract;
mod doubles;
mod memory_store;

pub use doubles::{
    EchoCredentials, FixedClock, FixedRandom, ScriptedHelper, ScriptedLauncher, oid, task_caps,
};
pub use memory_store::MemoryStore;

pub mod prelude {
    pub use crate::ports::{
        CommitRepository, ConflictRepository, NoticeRepository, ObjectRepository,
        RemoteTrackingRepository, StageRepository,
    };
}
