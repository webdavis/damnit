use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Memory {
    pub(crate) reported: BTreeMap<String, i64>,
}

impl Memory {
    pub(crate) fn read(since: Option<&str>) -> Memory {
        since
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    pub(crate) fn token(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub(crate) fn vanished(&self, seen: &BTreeSet<String>, window_start: i64) -> Vec<String> {
        self.reported
            .iter()
            .filter(|(id, end)| !seen.contains(*id) && **end > window_start)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests;
