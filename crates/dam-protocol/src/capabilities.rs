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
}

impl Capabilities {
    pub fn supported(&self) -> bool {
        self.protocol <= PROTOCOL_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_version_and_every_older_one_is_supported() {
        for protocol in 0..=PROTOCOL_VERSION {
            assert!(caps(protocol).supported(), "{protocol}");
        }
    }

    #[test]
    fn one_version_past_the_current_one_is_not_supported() {
        assert!(!caps(PROTOCOL_VERSION + 1).supported());
    }

    fn caps(protocol: u32) -> Capabilities {
        Capabilities {
            protocol,
            kinds: vec![],
            fields: vec![],
            credentials: vec![],
            incremental: false,
        }
    }
}
