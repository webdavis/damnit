pub mod contract;
mod debug_form;
mod doubles;
mod faulty_store;
mod filter_functions;
mod fixture;
mod holders;
mod launchers;
mod memory_store;
mod scenario;

pub use debug_form::each_field_in_its_debug_form;
pub use doubles::{
    DeclaredForm, EchoCredentials, FixedClock, FixedRandom, ScriptedHelper, ScriptedLauncher, oid,
    task_caps,
};
pub use faulty_store::FaultyStore;
pub use filter_functions::FakeFilterFunctions;
pub use fixture::{Fixture, remotes_launched_by};
pub use holders::{allowed, holder};
pub use launchers::{
    Pushed, accepting_all, answering, pulling, pulling_events, pushing,
    pushing_to_a_remote_that_removes_only_empty_containers, refusing_all, refusing_all_unasked,
};
pub use memory_store::MemoryStore;
pub use scenario::{Scenario, config, event, task, task_in, todoist};

pub mod prelude {
    pub use crate::ports::{
        AcceptedKindsRepository, CommitRepository, ConflictRepository, ItemRepository,
        NoticeRepository, ObjectRepository, RemoteTrackingRepository, StageRepository,
    };
}
