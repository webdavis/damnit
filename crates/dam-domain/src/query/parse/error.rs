use std::fmt;

use super::MAX_RECURSIVE_DESCENT_DEPTH;

#[derive(Debug, PartialEq, Eq)]
pub enum QueryError {
    Empty,
    Unexpected(String),
    MissingOperator(String),
    EndsEarly,
    BadValue { key: String, value: String },
    Unclosed,
    TooDeep,
    UnknownCategory(String),
    NotAValue { category: String, value: String },
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::Empty => f.write_str("the query is empty"),
            QueryError::Unexpected(t) => write!(f, "unexpected {t} in query"),
            QueryError::MissingOperator(t) => write!(f, "expected & or | before {t}"),
            QueryError::EndsEarly => f.write_str("the query ends where a term was expected"),
            QueryError::BadValue { key, value } => write!(f, "{key}: cannot read {value:?}"),
            QueryError::Unclosed => f.write_str("a ( has no matching )"),
            QueryError::TooDeep => write!(
                f,
                "the query nests deeper than {MAX_RECURSIVE_DESCENT_DEPTH}"
            ),
            QueryError::UnknownCategory(n) => write!(f, "{n:?} is not a declared category"),
            QueryError::NotAValue { category, value } => {
                write!(f, "{value} is not a value of category {category}")
            }
        }
    }
}

impl std::error::Error for QueryError {}
