mod base_url;
mod credential;
mod http;
mod remote;
mod secret;
mod serve;
mod service_error;

pub use base_url::{base_url, loopback_base};
pub use credential::credential;
pub use http::{Answer, agent, read, read_answer};
pub use remote::Remote;
pub use secret::Secret;
pub use serve::serve;
pub use service_error::{MAX_ERROR_BODY_CHARS, ServiceError, one_line};
