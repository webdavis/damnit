use dam_protocol::credential_variable;

use crate::Secret;

pub fn credential(
    remote: &str,
    name: &str,
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<Secret, String> {
    let variable = credential_variable(remote, name);
    lookup(&variable)
        .filter(|value| !value.is_empty())
        .map(Secret::from)
        .ok_or_else(|| format!("{variable} is not set; declare {name} under [remote.{remote}]"))
}

#[cfg(test)]
mod tests;
