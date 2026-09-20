//! Test doubles and the repository contract suite. Behind the `testing`
//! feature so `dam-adapters` can run the same contract against `SqliteStore`.

pub mod contract;
mod doubles;
mod memory_store;

pub use doubles::{
    EchoCredentials, FixedClock, FixedRandom, ScriptedHelper, ScriptedLauncher, oid, task_caps,
};
pub use memory_store::MemoryStore;

/// Every repository trait at once, so a test can read and write the store
/// directly without naming the family each call belongs to.
pub mod prelude {
    pub use crate::ports::{
        CommitRepository, ConflictRepository, NoticeRepository, ObjectRepository,
        RemoteTrackingRepository, StageRepository,
    };
}
