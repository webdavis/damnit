use std::cell::RefCell;
use std::collections::BTreeMap;

use dam_application::{LastRun, RunRecords, ScheduleError};

#[derive(Default)]
pub(crate) struct MemoryRunRecords(pub(crate) RefCell<BTreeMap<String, LastRun>>);

impl RunRecords for MemoryRunRecords {
    fn record(&self, remote: &str, run: &LastRun) -> Result<(), ScheduleError> {
        self.0.borrow_mut().insert(remote.into(), run.clone());
        Ok(())
    }
    fn last(&self, remote: &str) -> Result<Option<LastRun>, ScheduleError> {
        Ok(self.0.borrow().get(remote).cloned())
    }
}
