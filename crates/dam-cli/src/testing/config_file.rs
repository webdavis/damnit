use std::cell::RefCell;
use std::rc::Rc;

use dam_application::{Config, ConfigError, ConfigFile, LoadedConfig, NewToken, RemoteName};

use super::MissingCredentials;

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
    fn load(&self) -> Result<LoadedConfig, ConfigError> {
        Ok(LoadedConfig {
            config: Config::default(),
            credentials: Box::new(MissingCredentials),
        })
    }
    fn add_remote(&self, name: &RemoteName, url: &str) -> Result<Option<String>, ConfigError> {
        self.added
            .borrow_mut()
            .push((name.0.clone(), url.to_string()));
        Ok(None)
    }
    fn add_token(&self, _: &NewToken) -> Result<Option<String>, ConfigError> {
        Ok(None)
    }
    fn remove_token(&self, _: &str) -> Result<Option<String>, ConfigError> {
        Ok(None)
    }
}
