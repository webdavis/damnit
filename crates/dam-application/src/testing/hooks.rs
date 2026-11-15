use std::cell::RefCell;

use dam_domain::Object;

use crate::config::HookEvent;
use crate::ports::{FunctionFailure, Histories, History, HookFailure, HookWarnings, Hooks};

#[derive(Default)]
pub struct FakeHooks {
    pub failing: RefCell<Option<FunctionFailure>>,
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
    ) -> Result<(), FunctionFailure> {
        self.ran
            .borrow_mut()
            .push((event, task.base().subject.clone()));
        self.histories.borrow_mut().push(histories.of(task.oid()));
        match self.failing.borrow_mut().take() {
            Some(failure) => Err(failure),
            None => Ok(()),
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
