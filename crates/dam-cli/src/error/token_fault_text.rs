use std::fmt;

use dam_application::{TokenFault, TokenWarning};

use super::misread_text::MisreadText;

pub(crate) struct TokenFaultText<'a>(pub(crate) &'a TokenFault);

impl fmt::Display for TokenFaultText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TokenFault::Misread(misread) => write!(f, "{}", MisreadText(misread)),
            TokenFault::NoValue => f.write_str("it has no value"),
            TokenFault::ValueNotText => f.write_str("its value must be a string"),
            TokenFault::ValueNotAToken => f.write_str("its value is not a damnit token"),
            TokenFault::LimitsNotEnforced => f.write_str("its limits are not enforced yet"),
            TokenFault::Reserved(word) => write!(f, "{word} is reserved and cannot be granted"),
            TokenFault::NewerSchema { found, reads } => write!(
                f,
                "secrets.lua is schema {found}, and this dam reads schema {reads}"
            ),
            TokenFault::SchemaNotWhole { found } => write!(
                f,
                "schema in secrets.lua must be a whole number, and it is {found}"
            ),
            TokenFault::SharesValue { with } => {
                let tokens: Vec<String> = with.iter().map(|name| format!("token {name}")).collect();
                write!(f, "it holds the same value as {}", tokens.join(", "))
            }
        }
    }
}

pub(crate) struct TokenWarningText<'a>(pub(crate) &'a TokenWarning);

impl fmt::Display for TokenWarningText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TokenWarning::EveryTokenIgnored(misread) => {
                write!(f, "every token is ignored: {}", MisreadText(misread))
            }
            TokenWarning::Ignored { name, why } => {
                write!(f, "token {name} is ignored: {}", TokenFaultText(why))
            }
            TokenWarning::UnknownPermission { name, word } => {
                write!(f, "token {name}: unknown permission {word:?} is skipped")
            }
            TokenWarning::UnknownChannel { name, word } => {
                write!(f, "token {name}: unknown channel {word:?} is skipped")
            }
        }
    }
}

#[cfg(test)]
mod tests;
