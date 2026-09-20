//! The line protocol between dam and a remote helper.
//!
//! One JSON object per line in each direction, over the helper's standard
//! input and standard output. `docs/specs/protocol.md` in this crate is the
//! contract a third-party helper is written against; the compatibility policy
//! is these three rules:
//!
//! 1. Unknown fields are ignored, so a helper may add one and an older dam
//!    reads the rest of the response.
//! 2. Unknown message kinds are refused. A response matching no known shape is
//!    a protocol error rather than an empty answer.
//! 3. A helper declaring a version above [`PROTOCOL_VERSION`] is refused by
//!    name. The version only grows, so a helper written against version 1
//!    keeps working against every later dam.
//!
//! A line longer than [`MAX_LINE`] bytes is refused. The budget is per line,
//! so the length of a conversation is not bounded.

mod capabilities;
mod codec;
mod messages;

pub use capabilities::{Capabilities, PROTOCOL_VERSION};
pub use codec::{MAX_LINE, read_line, write_line};
pub use messages::{
    Mutation, MutationResult, PullResponse, PushResponse, Request, Response, WireAttachment,
    WireAttendee, WireConference, WireEvent, WireObject, WirePerson, WireTask,
};
