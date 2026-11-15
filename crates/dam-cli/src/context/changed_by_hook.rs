use std::cell::RefCell;

use dam_application::{HookEdits, HookEvent};
use dam_domain::Oid;

#[derive(Default)]
pub(crate) struct ChangedByHook(RefCell<Vec<(HookEvent, Oid)>>);

impl HookEdits for ChangedByHook {
    fn edited(&self, event: HookEvent, task: &Oid) {
        self.0.borrow_mut().push((event, task.clone()));
    }
}

impl ChangedByHook {
    pub(crate) fn all(&self) -> Vec<(HookEvent, Oid)> {
        self.0.borrow().clone()
    }
}
