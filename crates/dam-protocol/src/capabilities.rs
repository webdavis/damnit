use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

/// What a helper accepts. A helper never receives a field it did not declare,
/// and a pulled object never overwrites a field the helper did not declare.
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
}
