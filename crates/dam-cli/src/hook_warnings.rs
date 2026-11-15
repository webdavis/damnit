use dam_application::{HookEvent, HookFailure, HookWarnings};

use crate::error::HookText;

pub(crate) struct HookWarningsOnStandardError;

impl HookWarnings for HookWarningsOnStandardError {
    fn warn(&self, failure: &HookFailure) {
        eprintln!("{}", warning(failure));
    }
}

fn warning(failure: &HookFailure) -> String {
    let what_stays = match failure.event {
        HookEvent::TaskPulled => "the pull is kept",
        HookEvent::TaskNew | HookEvent::TaskChanged | HookEvent::TaskDone => "the change stands",
    };
    format!("warning: {}; {what_stays}", HookText(failure))
}

#[cfg(test)]
mod tests;
