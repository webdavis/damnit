pub mod config;
pub mod credentials;
pub mod errors;
pub mod merge;
pub mod ports;
#[cfg(test)]
pub(crate) mod testing;
pub mod use_cases;
pub mod wire;

pub use config::{Config, CredentialSpec, FilterConfig, RemoteConfig};
pub use credentials::resolve_credentials;
pub use dam_protocol::{
    Capabilities, Mutation, MutationResult, PullResponse, PushResponse, WireObject,
};
pub use errors::{Refusal, UseCaseError};
pub use merge::merge_fields;
pub use ports::{
    Clock, Conflict, CredentialError, CredentialSource, EditorError, EditorSession, HelperError,
    HelperLauncher, Notice, ObjectStore, Randomness, RemoteHelper, RemoteName, StoreError,
};
pub use use_cases::commit::{commit, log};
pub use use_cases::complete::{CompletePlan, Completed, Dispositions, complete, plan_complete};
pub use use_cases::edit::{EditFields, apply, edit};
pub use use_cases::list::list;
pub use use_cases::new_object::{NewEvent, NewTask, new_event, new_task};
pub use use_cases::relocate::relocate;
pub use use_cases::remove::{RemovePlan, plan_remove, remove};
pub use use_cases::stage::{add, add_all, reset};
pub use use_cases::status::{Status, diff_staged, diff_working, status};
