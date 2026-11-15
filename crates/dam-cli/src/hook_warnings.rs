use dam_application::{HookEvent, HookFailure, HookWarnings};

use crate::error::HookText;

pub(crate) struct HookWarningsOnStandardError;

impl HookWarnings for HookWarningsOnStandardError {
    fn warn(&self, failure: &HookFailure) {
        let what_stays = match failure.event {
            HookEvent::TaskPulled => "the pull is kept",
            HookEvent::TaskNew | HookEvent::TaskChanged | HookEvent::TaskDone => {
                "the change stands"
            }
        };
        eprintln!("warning: {}; {what_stays}", HookText(failure));
    }
}
