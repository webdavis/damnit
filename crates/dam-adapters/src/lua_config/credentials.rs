use dam_application::{CredentialSpec, Expected, Misread, RemoteName, Secret};
use mlua::Value;

use super::shape::text;
use super::strict::StrictMap;

pub(super) fn read_credentials(
    remote: &str,
    credentials: &StrictMap,
) -> Result<Vec<CredentialSpec>, Misread> {
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
                .ok_or_else(|| credentials.wrong(name, Expected::TextOrFunction)),
        })
        .collect()
}
