use dam_helper::{base_url, loopback_base};

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
        let base = loopback_base(BASE_URL_VARIABLE, base)?;
        Ok(Endpoints {
            authorization: format!("{base}/o/oauth2/v2/auth"),
            token: format!("{base}/token"),
            calendar: format!("{base}/calendar/v3"),
        })
    }

    pub fn from_env() -> Result<Endpoints, String> {
        match base_url(BASE_URL_VARIABLE)? {
            Some(base) => Endpoints::loopback(&base),
            None => Ok(Endpoints::production()),
        }
    }
}

#[cfg(test)]
mod tests;
