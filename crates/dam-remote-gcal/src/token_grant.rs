use crate::Secret;
use crate::api_error::SERVICE;
use crate::encoding::form;
use crate::oauth_error::oauth_error_code;

#[derive(Debug, PartialEq, Eq)]
pub struct Refusal {
    pub status: Option<u16>,
    pub code: Option<&'static str>,
}

impl Refusal {
    pub(crate) fn granted_without_the_field(&self) -> bool {
        self.status.is_some_and(granted)
    }
}

fn granted(status: u16) -> bool {
    (200..300).contains(&status)
}

pub(crate) fn token_grant(
    agent: &ureq::Agent,
    endpoint: &str,
    fields: &[(&str, &str)],
    wanted: &str,
) -> Result<Secret, Refusal> {
    let unreadable = || Refusal {
        status: None,
        code: None,
    };
    let response = agent
        .post(endpoint)
        .header("content-type", "application/x-www-form-urlencoded")
        .send(form(fields).as_str())
        .map_err(|_| unreadable())?;
    let answer = dam_helper::read(SERVICE, response).map_err(|_| unreadable())?;
    if !granted(answer.status) {
        return Err(Refusal {
            status: Some(answer.status),
            code: oauth_error_code(&answer.body),
        });
    }
    let value: serde_json::Value = serde_json::from_str(&answer.body).map_err(|_| unreadable())?;
    value
        .get(wanted)
        .and_then(serde_json::Value::as_str)
        .filter(|t| !t.is_empty())
        .map(Secret::from)
        .ok_or(Refusal {
            status: Some(answer.status),
            code: None,
        })
}
