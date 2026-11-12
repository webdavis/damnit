use std::fmt;

pub const CALLER_ID_CAP: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CallerId(String);

#[derive(Debug, PartialEq, Eq)]
pub enum CallerIdError {
    Empty,
    TooLong,
    LineBreak,
}

impl CallerId {
    pub fn parse(text: &str) -> Result<CallerId, CallerIdError> {
        if text.is_empty() {
            return Err(CallerIdError::Empty);
        }
        if text.chars().count() > CALLER_ID_CAP {
            return Err(CallerIdError::TooLong);
        }
        if text
            .chars()
            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
        {
            return Err(CallerIdError::LineBreak);
        }
        Ok(CallerId(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CallerIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallerIdError::Empty => f.write_str("an id cannot be empty"),
            CallerIdError::TooLong => write!(f, "an id holds at most {CALLER_ID_CAP} characters"),
            CallerIdError::LineBreak => {
                f.write_str("an id is one line, with no line breaks or control characters")
            }
        }
    }
}

impl std::error::Error for CallerIdError {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IdHolder {
    Owner,
    Token(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IdKey {
    pub holder: IdHolder,
    pub session: Option<String>,
    pub id: CallerId,
}

#[cfg(test)]
mod tests;
