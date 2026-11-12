use super::{Change, Details};
use crate::{Channel, KeyId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenAuthor {
    pub name: String,
    pub key_id: KeyId,
    pub channel: Channel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Author {
    Owner,
    Token(TokenAuthor),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Authored {
    pub change: Change,
    pub author: Author,
    pub details: Option<Details>,
}

impl Authored {
    pub fn by_owner(change: Change) -> Authored {
        Authored {
            change,
            author: Author::Owner,
            details: None,
        }
    }

    pub fn with_details(self, details: Option<&Details>) -> Authored {
        Authored {
            details: details.cloned(),
            ..self
        }
    }
}

impl Author {
    pub fn token(&self) -> Option<&TokenAuthor> {
        match self {
            Author::Owner => None,
            Author::Token(token) => Some(token),
        }
    }
}
