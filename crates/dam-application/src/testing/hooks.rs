use std::cell::RefCell;

use dam_domain::{Object, Oid};

use crate::config::HookEvent;
use crate::errors::Refusal;
use crate::ports::{
    EditRequest, EditRequests, FunctionFailure, Histories, History, HookFailure, HookWarnings,
    Hooks,
};
use crate::use_cases::edit::EditFields;

#[derive(Default)]
pub struct FakeHooks {
    pub failing: RefCell<Option<FunctionFailure>>,
    pub requesting: RefCell<Option<EditRequest>>,
    pub ran: RefCell<Vec<(HookEvent, String)>>,
    pub histories: RefCell<Vec<History>>,
}

impl FakeHooks {
    pub fn ran(&self) -> Vec<(HookEvent, String)> {
        self.ran.borrow().clone()
    }
}

impl Hooks for FakeHooks {
    fn run(
        &self,
        event: HookEvent,
        task: &Object,
        histories: &dyn Histories,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.ran
            .borrow_mut()
            .push((event, task.base().subject.clone()));
        self.histories.borrow_mut().push(histories.of(task.oid()));
        match self.failing.borrow_mut().take() {
            Some(failure) => Err(failure),
            None => Ok(self.requesting.borrow_mut().take()),
        }
    }
}

#[derive(Default)]
pub struct FakeHookWarnings {
    pub warned: RefCell<Vec<HookFailure>>,
}

impl HookWarnings for FakeHookWarnings {
    fn warn(&self, failure: &HookFailure) {
        self.warned.borrow_mut().push(failure.clone());
    }
}

#[derive(Default)]
pub struct FakeEditRequests {
    pub reading: RefCell<Option<Result<EditFields, String>>>,
    pub read: RefCell<Vec<(Oid, EditRequest)>>,
}

impl EditRequests for FakeEditRequests {
    fn fields(&self, task: &Oid, request: &EditRequest) -> Result<EditFields, String> {
        self.read.borrow_mut().push((task.clone(), request.clone()));
        self.reading
            .borrow_mut()
            .take()
            .unwrap_or_else(|| Ok(EditFields::default()))
    }

    fn refused(&self, refusal: &Refusal) -> String {
        format!("{refusal:?}")
    }
}
