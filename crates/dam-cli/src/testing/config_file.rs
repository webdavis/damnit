use std::cell::RefCell;
use std::rc::Rc;

use dam_application::{
    Config, ConfigError, ConfigFile, LoadedConfig, NewToken, RemoteName, TokenName,
};

use super::MissingCredentials;

#[derive(Clone, Default)]
pub(crate) struct FakeConfigFile {
    added: Rc<RefCell<Vec<(String, String)>>>,
    tokens_added: Rc<RefCell<Vec<String>>>,
    token_refusal: Option<String>,
}
impl FakeConfigFile {
    pub(crate) fn refusing_tokens(why: &str) -> FakeConfigFile {
        FakeConfigFile {
            token_refusal: Some(why.to_string()),
            ..FakeConfigFile::default()
        }
    }
    pub(crate) fn added(&self) -> Vec<(String, String)> {
        self.added.borrow().clone()
    }
    pub(crate) fn tokens_added(&self) -> Vec<String> {
        self.tokens_added.borrow().clone()
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
    fn can_add_token(&self, _: &TokenName) -> Result<(), ConfigError> {
        match &self.token_refusal {
            Some(why) => Err(ConfigError::Invalid(why.clone())),
            None => Ok(()),
        }
    }
    fn add_token(&self, token: &NewToken) -> Result<Option<String>, ConfigError> {
        self.can_add_token(&token.name)?;
        self.tokens_added
            .borrow_mut()
            .push(token.name.as_str().to_string());
        Ok(None)
    }
    fn remove_token(&self, _: &str) -> Result<Option<String>, ConfigError> {
        Ok(None)
    }
}
