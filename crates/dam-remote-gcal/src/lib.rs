//! The Google Calendar remote helper for dam. Read-only: it pulls events and
//! never creates, changes or deletes one.

pub mod endpoints;
pub mod secret;

pub use endpoints::{BASE_URL_VARIABLE, Endpoints};
pub use secret::Secret;
