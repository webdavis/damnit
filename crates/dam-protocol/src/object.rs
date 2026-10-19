use serde::de::{DeserializeOwned, Error as _};
use serde_json::Value;

pub(crate) fn read_object<T: DeserializeOwned>(value: Value) -> Result<T, serde_json::Error> {
    match value.is_object() {
        true => serde_json::from_value(value),
        false => Err(serde_json::Error::custom("expected a JSON object")),
    }
}
