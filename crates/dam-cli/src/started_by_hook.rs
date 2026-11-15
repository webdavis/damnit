use std::ffi::OsString;

use clap::CommandFactory;
use dam_application::{Refusal, STARTED_BY_HOOK_VARIABLE};

use crate::args::Cli;
use crate::output::Format;
use crate::request::Refused;

pub(crate) fn refuse_a_dam_a_hook_started(line: &[OsString]) -> Result<(), Refused> {
    let Some(event) = std::env::var_os(STARTED_BY_HOOK_VARIABLE) else {
        return Ok(());
    };
    let given = Cli::command()
        .ignore_errors(true)
        .try_get_matches_from(line)
        .ok();
    let flag = |name| given.as_ref().is_some_and(|given| given.get_flag(name));
    Err(Refused {
        format: Format::of(flag("json"), flag("toon")),
        error: Refusal::StartedByHook(event.to_string_lossy().into_owned()).into(),
    })
}
