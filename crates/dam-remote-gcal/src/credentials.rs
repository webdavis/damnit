use dam_helper::credential;

use crate::{Client, Secret};

#[derive(Debug)]
pub struct Credentials {
    pub client: Client,
    pub refresh_token: Secret,
}

impl Credentials {
    pub fn from_env(remote: &str) -> Result<Credentials, String> {
        Credentials::from_lookup(remote, |variable| std::env::var(variable).ok())
    }

    pub fn from_lookup(
        remote: &str,
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Result<Credentials, String> {
        let read = |name: &str| credential(remote, name, &lookup);
        Ok(Credentials {
            client: Client {
                id: read("client_id")?.expose().to_string(),
                secret: read("client_secret")?,
            },
            refresh_token: read("refresh_token")?,
        })
    }
}

#[cfg(test)]
mod tests;
