use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

pub(super) struct MatchFunction;

impl Serialize for MatchFunction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("function", &true)?;
        map.end()
    }
}
