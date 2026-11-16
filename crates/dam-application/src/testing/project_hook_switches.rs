use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::ports::{ConfigError, ProjectHookSwitches};

#[derive(Default)]
pub struct FakeProjectHookSwitches {
    pub switches: RefCell<BTreeMap<PathBuf, bool>>,
}

impl ProjectHookSwitches for FakeProjectHookSwitches {
    fn switch(&self, project: &Path) -> Result<Option<bool>, ConfigError> {
        Ok(self.switches.borrow().get(project).copied())
    }

    fn turn(&self, project: &Path, on: Option<bool>) -> Result<(), ConfigError> {
        let mut switches = self.switches.borrow_mut();
        match on {
            Some(on) => switches.insert(project.to_path_buf(), on),
            None => switches.remove(project),
        };
        Ok(())
    }
}
