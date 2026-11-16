use dam_application::{HookEvent, HookFailure, HookWarnings};

use crate::error::HookText;
use crate::output::Format;

pub(crate) struct HookWarningsOnStandardError {
    shown: bool,
}

impl HookWarningsOnStandardError {
    pub(crate) fn for_format(format: Format) -> HookWarningsOnStandardError {
        HookWarningsOnStandardError {
            shown: format != Format::Quiet,
        }
    }

    fn line(&self, failure: &HookFailure) -> Option<String> {
        self.shown.then(|| warning(failure))
    }
}

impl HookWarnings for HookWarningsOnStandardError {
    fn warn(&self, failure: &HookFailure) {
        if let Some(line) = self.line(failure) {
            eprintln!("{line}");
        }
    }

    fn warn_change_dropped(&self, failure: &HookFailure) {
        if self.shown {
            eprintln!("{}", dropped(failure));
        }
    }
}

fn dropped(failure: &HookFailure) -> String {
    format!(
        "warning: {}; that task is left as it was",
        HookText(failure)
    )
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
