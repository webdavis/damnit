use std::cell::RefCell;
use std::rc::Rc;

use dam_application::{Config, ConfigError, ConfigFile, RemoteName};

#[derive(Clone, Default)]
pub(crate) struct FakeConfigFile {
    added: Rc<RefCell<Vec<(String, String)>>>,
}
impl FakeConfigFile {
    pub(crate) fn added(&self) -> Vec<(String, String)> {
        self.added.borrow().clone()
    }
}
impl ConfigFile for FakeConfigFile {
    fn load(&self) -> Result<Config, ConfigError> {
        Ok(Config::default())
    }
    fn add_remote(&self, name: &RemoteName, url: &str) -> Result<Option<String>, ConfigError> {
        self.added
            .borrow_mut()
            .push((name.0.clone(), url.to_string()));
        Ok(None)
    }
}
