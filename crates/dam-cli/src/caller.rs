use std::ffi::OsString;

use dam_application::{Access, Config, Owner, Refusal, authenticate};
use dam_domain::{Channel, Timestamp};

use crate::error::CliError;

pub(crate) fn caller(
    config: &Config,
    presented: Option<OsString>,
    now: Timestamp,
) -> Result<Box<dyn Access>, CliError> {
    let Some(presented) = presented else {
        return Ok(Box::new(Owner));
    };
    let text = presented.to_str().unwrap_or_default();
    let holder = authenticate(&config.tokens, text, Channel::Local, now).map_err(Refusal::Token)?;
    Ok(Box::new(holder))
}
