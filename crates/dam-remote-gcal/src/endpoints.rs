use std::env::VarError;

pub const BASE_URL_VARIABLE: &str = "DAM_GCAL_BASE_URL";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoints {
    pub authorization: String,
    pub token: String,
    pub calendar: String,
}

impl Endpoints {
    pub fn production() -> Endpoints {
        Endpoints {
            authorization: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token: "https://oauth2.googleapis.com/token".into(),
            calendar: "https://www.googleapis.com/calendar/v3".into(),
        }
    }

    pub fn loopback(base: &str) -> Result<Endpoints, String> {
        let base = loopback_base_only(base)?;
        Ok(Endpoints {
            authorization: format!("{base}/o/oauth2/v2/auth"),
            token: format!("{base}/token"),
            calendar: format!("{base}/calendar/v3"),
        })
    }

    pub fn from_env() -> Result<Endpoints, String> {
        Endpoints::from_variable(std::env::var(BASE_URL_VARIABLE))
    }

    fn from_variable(value: Result<String, VarError>) -> Result<Endpoints, String> {
        match value {
            Ok(base) => Endpoints::loopback(&base),
            Err(VarError::NotPresent) => Ok(Endpoints::production()),
            Err(VarError::NotUnicode(_)) => Err(format!(
                "{}; the value it holds is not Unicode",
                seam_rule()
            )),
        }
    }
}

fn seam_rule() -> String {
    format!(
        "{BASE_URL_VARIABLE} is a test seam and must be http://127.0.0.1:<port> or \
         http://localhost:<port>"
    )
}

fn loopback_base_only(value: &str) -> Result<String, String> {
    let refused = || format!("{}, not {value:?}", seam_rule());
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

#[cfg(test)]
mod tests;
