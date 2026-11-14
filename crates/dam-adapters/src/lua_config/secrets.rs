use std::collections::BTreeMap;

use dam_application::{Config, ConfigError, CredentialSpec, Tokens};

use super::credentials::read_credentials;
use super::strict::StrictMap;
use super::token::read_tokens;

const SECRETS: [&str; 3] = ["schema", "remote", "token"];

pub(super) struct Secrets {
    credentials: BTreeMap<String, Vec<CredentialSpec>>,
    tokens: Tokens,
}

impl Secrets {
    pub(super) fn complete(
        self,
        config: &mut Config,
        config_name: &str,
    ) -> Result<(), ConfigError> {
        for (name, specs) in self.credentials {
            let remote = config
                .remotes
                .iter_mut()
                .find(|r| r.name.0 == name)
                .ok_or_else(|| ConfigError::UndeclaredRemote {
                    remote: name.clone(),
                    config: config_name.to_string(),
                })?;
            remote.credentials = specs;
        }
        config.tokens = self.tokens;
        Ok(())
    }
}

pub(super) fn read_secrets(root: &StrictMap) -> Result<Secrets, ConfigError> {
    root.only(&SECRETS)?;
    let credentials = root
        .maps_in("remote")?
        .iter()
        .map(|(name, remote)| Ok((name.clone(), read_credentials(name, remote)?)))
        .collect::<Result<_, ConfigError>>()?;
    Ok(Secrets {
        credentials,
        tokens: read_tokens(root),
    })
}
