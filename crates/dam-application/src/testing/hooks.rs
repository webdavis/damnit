use std::cell::RefCell;

use dam_domain::{Oid, WorkedOut};

use crate::config::HookEvent;
use crate::errors::Refusal;
use crate::ports::{
    EditRequest, EditRequests, FunctionFailure, Handed, Histories, History, HookEdits, HookFailure,
    HookWarnings, Hooks, Pull,
};
use crate::use_cases::edit::EditFields;

#[derive(Default)]
pub struct FakeHooks {
    pub failing: RefCell<Option<FunctionFailure>>,
    pub failing_every_call: RefCell<Option<FunctionFailure>>,
    pub requesting: RefCell<Option<EditRequest>>,
    pub ran: RefCell<Vec<(HookEvent, String)>>,
    pub histories: RefCell<Vec<History>>,
    pub pulled: RefCell<Vec<(String, Option<String>, Pull)>>,
    pub handed: RefCell<Vec<WorkedOut>>,
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
        task: &Handed,
        histories: &dyn Histories,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.handed.borrow_mut().push(task.worked_out.clone());
        let task = &task.object;
        self.ran
            .borrow_mut()
            .push((event, task.base().subject.clone()));
        self.histories.borrow_mut().push(histories.of(task.oid()));
        self.answer()
    }

    fn pulled(
        &self,
        task: &Handed,
        before: Option<&Handed>,
        pull: &Pull,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.handed.borrow_mut().push(task.worked_out.clone());
        let subject = |handed: &Handed| handed.object.base().subject.clone();
        self.pulled
            .borrow_mut()
            .push((subject(task), before.map(subject), pull.clone()));
        self.answer()
    }
}

impl FakeHooks {
    fn answer(&self) -> Result<Option<EditRequest>, FunctionFailure> {
        let every_call = self.failing_every_call.borrow().clone();
        match self.failing.borrow_mut().take().or(every_call) {
            Some(failure) => Err(failure),
            None => Ok(self.requesting.borrow_mut().take()),
        }
    }
}

#[derive(Default)]
pub struct FakeHookWarnings {
    pub warned: RefCell<Vec<HookFailure>>,
    pub dropped: RefCell<Vec<HookFailure>>,
}

impl HookWarnings for FakeHookWarnings {
    fn warn(&self, failure: &HookFailure) {
        self.warned.borrow_mut().push(failure.clone());
    }

    fn warn_change_dropped(&self, failure: &HookFailure) {
        self.dropped.borrow_mut().push(failure.clone());
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

#[derive(Default)]
pub struct FakeHookEdits {
    pub edited: RefCell<Vec<(HookEvent, Oid)>>,
}

impl HookEdits for FakeHookEdits {
    fn edited(&self, event: HookEvent, task: &Oid) {
        self.edited.borrow_mut().push((event, task.clone()));
    }
}
