mod error;
mod misread;

use crate::config::Config;
use crate::ports::{CredentialSource, RemoteName};
use crate::tokens::{NewToken, TokenName};

pub use error::{ByHand, ConfigError, EditFault, EditedTable, Uneditable};
pub use misread::{Expected, Misread, Place};

#[derive(Debug, PartialEq, Eq)]
pub struct SecretsTightened {
    pub was: u32,
}

pub struct LoadedConfig {
    pub config: Config,
    pub credentials: Box<dyn CredentialSource>,
}

pub trait ConfigFile {
    fn load(&self) -> Result<LoadedConfig, ConfigError>;
    fn add_remote(
        &self,
        name: &RemoteName,
        url: &str,
    ) -> Result<Option<SecretsTightened>, ConfigError>;
    fn can_add_token(&self, name: &TokenName) -> Result<(), ConfigError>;
    fn add_token(&self, token: &NewToken) -> Result<Option<SecretsTightened>, ConfigError>;
    fn remove_token(&self, name: &str) -> Result<Option<SecretsTightened>, ConfigError>;
}
