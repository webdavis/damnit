use dam_protocol::credential_variable;

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
        let read = |name: &str| {
            let variable = credential_variable(remote, name);
            lookup(&variable).filter(|v| !v.is_empty()).ok_or_else(|| {
                format!("{variable} is not set; declare {name} under [remote.{remote}]")
            })
        };
        Ok(Credentials {
            client: Client {
                id: read("client_id")?,
                secret: Secret::from(read("client_secret")?),
            },
            refresh_token: Secret::from(read("refresh_token")?),
        })
    }
}

#[cfg(test)]
mod tests;
