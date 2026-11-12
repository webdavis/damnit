use std::collections::BTreeMap;

use crate::{Channel, Timestamp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Details {
    pub meta: BTreeMap<String, String>,
    pub host: Option<String>,
    pub dir: Option<String>,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub at: Timestamp,
    pub channel: Channel,
}

impl Details {
    pub fn session(&self) -> Option<&str> {
        self.meta.get("session").map(String::as_str)
    }
}
