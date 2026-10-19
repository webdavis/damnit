use std::env::{self, VarError};

pub fn base_url(variable: &str) -> Result<Option<String>, String> {
    read_base(variable, env::var(variable))
}

pub fn loopback_base(variable: &str, value: &str) -> Result<String, String> {
    let refused = || format!("{}, not {value:?}", seam_rule(variable));
    let base = value.strip_suffix('/').unwrap_or(value);
    let (host, port) = base
        .strip_prefix("http://")
        .and_then(|rest| rest.rsplit_once(':'))
        .ok_or_else(refused)?;
    if !matches!(host, "127.0.0.1" | "localhost")
        || port.is_empty()
        || !port.chars().all(|c| c.is_ascii_digit())
    {
        return Err(refused());
    }
    Ok(base.to_string())
}

fn read_base(variable: &str, value: Result<String, VarError>) -> Result<Option<String>, String> {
    match value {
        Ok(base) => loopback_base(variable, &base).map(Some),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(format!(
            "{}; the value it holds is not Unicode",
            seam_rule(variable)
        )),
    }
}

fn seam_rule(variable: &str) -> String {
    format!(
        "{variable} is a test seam and must be http://127.0.0.1:<port> or http://localhost:<port>"
    )
}

#[cfg(test)]
mod tests;
