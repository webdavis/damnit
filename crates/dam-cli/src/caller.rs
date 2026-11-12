use std::collections::BTreeMap;
use std::ffi::OsString;

use dam_adapters::process_details;
use dam_application::{Access, Config, Detailed, Owner, Refusal, authenticate};
use dam_domain::{Channel, Timestamp};

use crate::error::CliError;

pub(crate) fn caller(
    config: &Config,
    presented: Option<OsString>,
    now: Timestamp,
    meta: &BTreeMap<String, String>,
) -> Result<Box<dyn Access>, CliError> {
    let details = process_details(meta.clone(), Channel::Local, now);
    let Some(presented) = presented else {
        return Ok(Box::new(Detailed {
            caller: Box::new(Owner),
            details,
        }));
    };
    let text = presented.to_str().unwrap_or_default();
    let holder =
        authenticate(&config.tokens, text, Channel::Local, now, meta).map_err(Refusal::Token)?;
    Ok(Box::new(Detailed {
        caller: Box::new(holder),
        details,
    }))
}
