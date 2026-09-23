//! The three credentials dam resolved and handed over in the environment,
//! under this remote's own name.

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

    /// An empty value is no value: dam never hands one over, so it is a
    /// variable set by something else.
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
mod tests {
    use super::*;

    use std::collections::HashMap;

    fn environment(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let held: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |name| held.get(name).cloned()
    }

    #[test]
    fn credentials_are_read_under_the_remotes_own_name() {
        let lookup = environment(&[
            ("DAM_WORK_CAL_CLIENT_ID", "id-1"),
            ("DAM_WORK_CAL_CLIENT_SECRET", "GOCSPX-S"),
            ("DAM_WORK_CAL_REFRESH_TOKEN", "1//0gR"),
        ]);
        let c = Credentials::from_lookup("work-cal", lookup).unwrap();
        assert_eq!(c.client.id, "id-1");
        assert_eq!(c.client.secret.expose(), "GOCSPX-S");
        assert_eq!(c.refresh_token.expose(), "1//0gR");
    }

    #[test]
    fn a_missing_credential_names_its_variable_and_its_config_key() {
        let lookup = environment(&[
            ("DAM_HALF_CLIENT_ID", "id-1"),
            ("DAM_HALF_CLIENT_SECRET", "GOCSPX-S"),
            ("DAM_HALF_REFRESH_TOKEN", ""),
        ]);
        let said = Credentials::from_lookup("half", lookup).unwrap_err();
        assert!(said.contains("DAM_HALF_REFRESH_TOKEN"), "{said}");
        assert!(said.contains("refresh_token under [remote.half]"), "{said}");
        assert!(!said.contains("GOCSPX-S"), "{said}");
    }
}
