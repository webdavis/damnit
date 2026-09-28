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
                "{BASE_URL_VARIABLE} is a test seam and must be http://127.0.0.1:<port> or \
                 http://localhost:<port>, not a value that is not Unicode"
            )),
        }
    }
}

fn loopback_base_only(value: &str) -> Result<String, String> {
    let refused = || {
        format!(
            "{BASE_URL_VARIABLE} is a test seam and must be http://127.0.0.1:<port> or \
             http://localhost:<port>, not {value:?}"
        )
    };
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
mod tests {
    use super::*;

    #[test]
    fn production_names_googles_three_hosts() {
        let e = Endpoints::production();
        assert_eq!(
            e.authorization,
            "https://accounts.google.com/o/oauth2/v2/auth"
        );
        assert_eq!(e.token, "https://oauth2.googleapis.com/token");
        assert_eq!(e.calendar, "https://www.googleapis.com/calendar/v3");
    }

    #[test]
    fn a_loopback_base_serves_all_three_under_one_port() {
        let e = Endpoints::loopback("http://127.0.0.1:8080/").unwrap();
        assert_eq!(e.authorization, "http://127.0.0.1:8080/o/oauth2/v2/auth");
        assert_eq!(e.token, "http://127.0.0.1:8080/token");
        assert_eq!(e.calendar, "http://127.0.0.1:8080/calendar/v3");
        assert!(Endpoints::loopback("http://localhost:9").is_ok());
    }

    #[test]
    fn a_base_url_that_is_not_unicode_is_refused_rather_than_read_as_unset() {
        let not_unicode = std::env::VarError::NotUnicode(std::ffi::OsString::from("x"));
        let err = Endpoints::from_variable(Err(not_unicode)).unwrap_err();
        assert!(err.contains(BASE_URL_VARIABLE), "{err}");
        assert_eq!(
            Endpoints::from_variable(Err(std::env::VarError::NotPresent)),
            Ok(Endpoints::production())
        );
        assert_eq!(
            Endpoints::from_variable(Ok("http://127.0.0.1:9".into())),
            Endpoints::loopback("http://127.0.0.1:9")
        );
    }

    #[test]
    fn the_seam_that_carries_credentials_refuses_every_address_but_loopback() {
        for refused in [
            "https://oauth2.googleapis.com",
            "http://evil.test:8080",
            "http://127.0.0.1.evil.test:8080",
            "http://user@127.0.0.1:8080",
            "http://localhost:8080@evil.test",
            "http://127.0.0.1",
            "http://127.0.0.1:",
            "https://127.0.0.1:8080",
            "",
        ] {
            let err = Endpoints::loopback(refused).unwrap_err();
            assert!(err.contains(BASE_URL_VARIABLE), "{refused}: {err}");
        }
    }
}
