use serde::de::{DeserializeOwned, Error as _};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Reply<T> {
    Failed { error: String },
    Answered(T),
}

#[derive(Deserialize)]
struct Failure {
    error: String,
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for Reply<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        if value.get("error").is_some() {
            let failure: Failure = serde_json::from_value(value).map_err(D::Error::custom)?;
            return Ok(Reply::Failed {
                error: failure.error,
            });
        }
        serde_json::from_value(value)
            .map(Reply::Answered)
            .map_err(D::Error::custom)
    }
}

impl<T> From<Result<T, String>> for Reply<T> {
    fn from(result: Result<T, String>) -> Reply<T> {
        match result {
            Ok(answer) => Reply::Answered(answer),
            Err(error) => Reply::Failed { error },
        }
    }
}

#[cfg(test)]
mod tests;
