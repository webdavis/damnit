pub mod contract;
mod doubles;
mod faulty_store;
mod fixture;
mod launchers;
mod memory_store;
mod scenario;

pub use doubles::{
    EchoCredentials, FixedClock, FixedRandom, ScriptedHelper, ScriptedLauncher, oid, task_caps,
};
pub use faulty_store::FaultyStore;
pub use fixture::{Fixture, remotes_launched_by};
pub use launchers::{
    Pushed, accepting_all, answering, pulling, pulling_events, pushing, refusing_all,
};
pub use memory_store::MemoryStore;
pub use scenario::{Scenario, config, task, todoist};

pub mod prelude {
    pub use crate::ports::{
        AcceptedKindsRepository, CommitRepository, ConflictRepository, NoticeRepository,
        ObjectRepository, RemoteTrackingRepository, StageRepository,
    };
}
