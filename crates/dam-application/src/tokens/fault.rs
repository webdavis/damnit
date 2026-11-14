use crate::ports::Misread;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenFault {
    Misread(Box<Misread>),
    NoValue,
    ValueNotText,
    ValueNotAToken,
    LimitsNotEnforced,
    Reserved(String),
    NewerSchema { found: i64, reads: i64 },
    SchemaNotWhole { found: String },
    SharesValue { with: Vec<String> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenWarning {
    EveryTokenIgnored(Misread),
    Ignored { name: String, why: TokenFault },
    UnknownPermission { name: String, word: String },
    UnknownChannel { name: String, word: String },
}

impl From<Misread> for TokenFault {
    fn from(misread: Misread) -> TokenFault {
        TokenFault::Misread(Box::new(misread))
    }
}
