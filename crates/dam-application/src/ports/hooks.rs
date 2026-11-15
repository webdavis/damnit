use dam_domain::Object;

use crate::config::HookEvent;
use crate::ports::{FunctionFailure, Histories};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookFailure {
    pub event: HookEvent,
    pub failure: FunctionFailure,
}

pub trait Hooks {
    fn run(
        &self,
        event: HookEvent,
        task: &Object,
        histories: &dyn Histories,
    ) -> Result<(), FunctionFailure>;
}

pub trait HookWarnings {
    fn warn(&self, failure: &HookFailure);
}
