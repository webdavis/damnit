use dam_application::{Expected, Misread};
use mlua::Value;

use super::strict::StrictMap;

pub(super) fn whole_number(map: &StrictMap, key: &str) -> Result<Option<usize>, Misread> {
    match map.get(key) {
        None => Ok(None),
        Some(Value::Integer(n)) => usize::try_from(*n)
            .map(Some)
            .map_err(|_| map.wrong(key, Expected::WholeNumber)),
        Some(_) => Err(map.wrong(key, Expected::WholeNumber)),
    }
}
