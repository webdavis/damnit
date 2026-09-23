//! The Google Calendar remote helper for dam. Read-only: it pulls events and
//! never creates, changes or deletes one.

pub(crate) mod encoding;
pub mod endpoints;
pub(crate) mod http;
pub(crate) mod oauth_error;
pub mod secret;
pub mod sign_in;

pub use endpoints::{BASE_URL_VARIABLE, Endpoints};
pub use secret::Secret;
pub use sign_in::{Client, SignIn, SignInError};
