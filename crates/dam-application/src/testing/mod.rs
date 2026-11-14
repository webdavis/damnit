pub mod contract;
mod doubles;
mod faulty_store;
mod fixture;
mod holders;
mod launchers;
mod memory_store;
mod scenario;

pub use doubles::{
    EchoCredentials, FixedClock, FixedRandom, ScriptedHelper, ScriptedLauncher, oid, task_caps,
};
pub use faulty_store::FaultyStore;
pub use fixture::{Fixture, remotes_launched_by};
pub use holders::{allowed, holder};
pub use launchers::{
    Pushed, accepting_all, answering, pulling, pulling_events, pushing,
    pushing_to_a_remote_that_removes_only_empty_containers, refusing_all,
};
pub use memory_store::MemoryStore;
pub use scenario::{Scenario, config, event, task, task_in, todoist};

pub mod prelude {
    pub use crate::ports::{
        AcceptedKindsRepository, CommitRepository, ConflictRepository, ItemRepository,
        NoticeRepository, ObjectRepository, RemoteTrackingRepository, StageRepository,
    };
}
