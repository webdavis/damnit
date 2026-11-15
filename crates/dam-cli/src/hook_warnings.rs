use dam_application::{HookFailure, HookWarnings};

use crate::error::HookText;

pub(crate) struct HookWarningsOnStandardError;

impl HookWarnings for HookWarningsOnStandardError {
    fn warn(&self, failure: &HookFailure) {
        eprintln!("warning: {}; the change stands", HookText(failure));
    }
}
