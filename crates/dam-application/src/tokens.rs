mod fault;

use std::collections::BTreeSet;

use dam_domain::{Channel, Grant, Timestamp, Token};

pub use fault::{TokenFault, TokenWarning};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tokens {
    pub names: Vec<String>,
    pub entries: Vec<TokenConfig>,
    pub warnings: Vec<TokenWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenConfig {
    pub name: String,
    pub value: Token,
    pub standing: Standing,
}

pub fn disable_shared_values(mut entries: Vec<TokenConfig>) -> Vec<TokenConfig> {
    let sharers: Vec<Vec<String>> = entries
        .iter()
        .map(|entry| {
            entries
                .iter()
                .filter(|other| other.name != entry.name && other.value == entry.value)
                .map(|other| other.name.clone())
                .collect()
        })
        .collect();
    for (entry, with) in entries.iter_mut().zip(sharers) {
        if !with.is_empty() {
            entry.standing = Standing::Disabled(TokenFault::SharesValue { with });
        }
    }
    entries
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Standing {
    Usable(TokenTerms),
    Disabled(TokenFault),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenTerms {
    pub expires: Option<Timestamp>,
    pub not_before: Option<Timestamp>,
    pub via: BTreeSet<Channel>,
    pub require: Vec<String>,
    pub grants: Vec<Grant>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenName(String);

impl TokenName {
    pub fn parse(text: &str) -> Result<TokenName, String> {
        let bare = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
        if text.is_empty() || !text.chars().all(bare) {
            return Err(format!(
                "token name {text:?} must be letters, digits, - and _"
            ));
        }
        Ok(TokenName(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewToken {
    pub name: TokenName,
    pub value: Token,
    pub expires: Option<Timestamp>,
    pub via: Option<BTreeSet<Channel>>,
    pub require: Vec<String>,
    pub grant: Grant,
}
