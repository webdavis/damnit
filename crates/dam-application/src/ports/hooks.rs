use std::path::PathBuf;

use dam_domain::{Object, Oid};

use crate::config::HookEvent;
use crate::errors::Refusal;
use crate::ports::{FunctionFailure, FunctionFault, Histories, Pull};
use crate::use_cases::edit::EditFields;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookFailure {
    pub event: HookEvent,
    pub failure: FunctionFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditRequest {
    pub json: String,
    pub file: PathBuf,
    pub line: Option<u32>,
}

impl EditRequest {
    pub fn failed(&self, why: String) -> FunctionFailure {
        FunctionFailure {
            file: self.file.clone(),
            line: self.line,
            fault: FunctionFault::Requested { why },
        }
    }
}

pub trait Hooks {
    fn run(
        &self,
        event: HookEvent,
        task: &Object,
        histories: &dyn Histories,
    ) -> Result<Option<EditRequest>, FunctionFailure>;

    fn pulled(
        &self,
        task: &Object,
        before: Option<&Object>,
        pull: &Pull,
    ) -> Result<Option<EditRequest>, FunctionFailure>;
}

pub trait EditRequests {
    fn fields(&self, task: &Oid, request: &EditRequest) -> Result<EditFields, String>;
    fn refused(&self, refusal: &Refusal) -> String;
}

pub trait HookEdits {
    fn edited(&self, event: HookEvent, task: &Oid);
}

pub trait HookWarnings {
    fn warn(&self, failure: &HookFailure);
    fn warn_change_dropped(&self, failure: &HookFailure);
}
