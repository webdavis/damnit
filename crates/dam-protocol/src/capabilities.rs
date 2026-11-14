use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    pub protocol: u32,
    #[serde(default)]
    pub kinds: Vec<String>,
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub credentials: Vec<String>,
    #[serde(default)]
    pub incremental: bool,
    #[serde(default)]
    pub removes_only_empty_containers: bool,
}

impl Capabilities {
    pub fn supported(&self) -> bool {
        (1..=PROTOCOL_VERSION).contains(&self.protocol)
    }
}

#[cfg(test)]
mod tests;
