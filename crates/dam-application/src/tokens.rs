use std::collections::BTreeSet;

use dam_domain::{Channel, Grant, Timestamp, Token};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tokens {
    pub entries: Vec<TokenConfig>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenConfig {
    pub name: String,
    pub value: Token,
    pub standing: Standing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Standing {
    Usable(TokenTerms),
    Disabled(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenTerms {
    pub expires: Option<Timestamp>,
    pub not_before: Option<Timestamp>,
    pub via: BTreeSet<Channel>,
    pub require: Vec<String>,
    pub grants: Vec<Grant>,
}
