use std::collections::BTreeMap;
use std::ffi::OsString;

use dam_adapters::process_details;
use dam_application::{Access, Config, Detailed, Holder, Owner, Refusal, authenticate};
use dam_domain::{Channel, Timestamp};

use crate::error::CliError;

pub(crate) fn authenticated(
    config: &Config,
    presented: Option<OsString>,
    now: Timestamp,
    meta: &BTreeMap<String, String>,
) -> Result<Option<Holder>, CliError> {
    let Some(presented) = presented else {
        return Ok(None);
    };
    let text = presented.to_str().unwrap_or_default();
    let holder =
        authenticate(&config.tokens, text, Channel::Local, now, meta).map_err(Refusal::Token)?;
    Ok(Some(holder))
}

pub(crate) fn caller(
    holder: Option<Holder>,
    now: Timestamp,
    meta: &BTreeMap<String, String>,
) -> Box<dyn Access> {
    let caller: Box<dyn Access> = match holder {
        Some(holder) => Box::new(holder),
        None => Box::new(Owner),
    };
    Box::new(Detailed {
        caller,
        details: process_details(meta.clone(), Channel::Local, now),
    })
}
