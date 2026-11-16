use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::ports::{ConfigError, ProjectHookSwitch, ProjectHookSwitches};

#[derive(Default)]
pub struct FakeProjectHookSwitches {
    pub switches: RefCell<BTreeMap<PathBuf, ProjectHookSwitch>>,
}

impl ProjectHookSwitches for FakeProjectHookSwitches {
    fn switch(&self, project: &Path) -> Result<Option<ProjectHookSwitch>, ConfigError> {
        Ok(self.switches.borrow().get(project).cloned())
    }

    fn turn(&self, project: &Path, switch: Option<ProjectHookSwitch>) -> Result<(), ConfigError> {
        let mut switches = self.switches.borrow_mut();
        match switch {
            Some(switch) => switches.insert(project.to_path_buf(), switch),
            None => switches.remove(project),
        };
        Ok(())
    }
}
