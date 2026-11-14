use dam_application::{ConfigError, CredentialSpec, RemoteName, Secret};
use mlua::Value;

use super::shape::text;
use super::strict::StrictMap;

pub(super) fn read_credentials(
    remote: &str,
    credentials: &StrictMap,
) -> Result<Vec<CredentialSpec>, ConfigError> {
    credentials
        .entries()
        .map(|(name, value)| match value {
            Value::Function(_) => Ok(CredentialSpec::Computed {
                remote: RemoteName(remote.to_string()),
                name: name.to_string(),
            }),
            _ => text(value)
                .map(|literal| CredentialSpec::Literal {
                    name: name.to_string(),
                    value: Secret::from(literal.as_str()),
                })
                .ok_or_else(|| {
                    ConfigError::Invalid(
                        credentials.refuse(format_args!("{name} must be a string or a function")),
                    )
                }),
        })
        .collect()
}
