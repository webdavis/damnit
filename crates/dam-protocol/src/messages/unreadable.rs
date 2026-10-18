use serde_json::Value;
use serde_path_to_error::Segment;

use crate::wire::WireObject;

pub(super) fn read_object(value: &Value) -> Result<WireObject, String> {
    serde_path_to_error::deserialize(value).map_err(|error| why(value, &error))
}

fn why(value: &Value, error: &serde_path_to_error::Error<serde_json::Error>) -> String {
    let message = error.inner().to_string();
    if let Some(field) = message
        .strip_prefix("missing field `")
        .and_then(|rest| rest.strip_suffix('`'))
    {
        return format!("{field}: missing");
    }
    let mut at = value;
    let mut field = None;
    for segment in error.path() {
        match segment {
            Segment::Map { key } => {
                at = &at[key.as_str()];
                field = Some(key.as_str());
            }
            Segment::Seq { index } => at = &at[*index],
            Segment::Enum { .. } | Segment::Unknown => {}
        }
    }
    match field {
        Some(field) => format!("{field}: cannot read {at}"),
        None => format!("cannot read {at}"),
    }
}
