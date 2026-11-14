use std::cell::RefCell;
use std::rc::Rc;

use dam_application::{
    ByHand, Config, ConfigError, ConfigFile, EditedTable, LoadedConfig, NewToken, RemoteName,
    SecretsTightened, TokenName, Uneditable,
};

use super::MissingCredentials;

#[derive(Clone, Default)]
pub(crate) struct FakeConfigFile {
    added: Rc<RefCell<Vec<(String, String)>>>,
    tokens_added: Rc<RefCell<Vec<String>>>,
    refuses_tokens: bool,
}
impl FakeConfigFile {
    pub(crate) fn refusing_tokens() -> FakeConfigFile {
        FakeConfigFile {
            refuses_tokens: true,
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
    fn add_remote(
        &self,
        name: &RemoteName,
        url: &str,
    ) -> Result<Option<SecretsTightened>, ConfigError> {
        self.added
            .borrow_mut()
            .push((name.0.clone(), url.to_string()));
        Ok(None)
    }
    fn can_add_token(&self, name: &TokenName) -> Result<(), ConfigError> {
        match self.refuses_tokens {
            true => Err(ConfigError::CannotEdit {
                table: EditedTable::Tokens,
                why: Uneditable::TableBuiltInCode,
                by_hand: ByHand::AddToken {
                    name: name.as_str().to_string(),
                },
            }),
            false => Ok(()),
        }
    }
    fn add_token(&self, token: &NewToken) -> Result<Option<SecretsTightened>, ConfigError> {
        self.can_add_token(&token.name)?;
        self.tokens_added
            .borrow_mut()
            .push(token.name.as_str().to_string());
        Ok(None)
    }
    fn remove_token(&self, _: &str) -> Result<Option<SecretsTightened>, ConfigError> {
        Ok(None)
    }
}
