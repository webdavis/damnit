pub mod contract;
mod debug_form;
mod doubles;
mod faulty_store;
mod filter_functions;
mod fixture;
mod holders;
mod hooks;
mod interruption;
mod launchers;
mod memory_store;
mod progress;
mod remote_fixture;
mod scenario;
mod settings_file;

pub use debug_form::each_field_in_its_debug_form;
pub use doubles::{
    DeclaredForm, EchoCredentials, FixedClock, FixedRandom, ScriptedHelper, ScriptedLauncher, oid,
    task_caps,
};
pub use faulty_store::FaultyStore;
pub use filter_functions::FakeFilterFunctions;
pub use fixture::{Fixture, remotes_launched_by};
pub use holders::{allowed, holder};
pub use hooks::{FakeEditRequests, FakeHookEdits, FakeHookWarnings, FakeHooks};
pub use interruption::FakeInterruption;
pub use launchers::{
    Pushed, accepting_all, answering, pulling, pulling_events, pushing,
    pushing_to_a_remote_that_removes_only_empty_containers, pushing_with, refusing_all,
    refusing_all_unasked,
};
pub use memory_store::MemoryStore;
pub use progress::{FakeProgress, Said};
pub use remote_fixture::{config, todoist};
pub use scenario::{Scenario, event, task, task_in};
pub use settings_file::{FakeSettingsFile, Written};

pub mod prelude {
    pub use crate::ports::{
        AcceptedKindsRepository, CommitRepository, ConflictRepository, ItemRepository,
        NoticeRepository, ObjectRepository, RemoteTrackingRepository, StageRepository,
    };
}

#[cfg(test)]
mod settings_file_tests;
