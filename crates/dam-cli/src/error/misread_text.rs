use std::fmt;

use dam_application::{Expected, Misread, Place};

pub(super) struct MisreadText<'a>(pub(super) &'a Misread);

impl fmt::Display for MisreadText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Misread::UnknownKey { key, at } => write!(f, "unknown key {key} {}", PlaceText(at)),
            Misread::UnknownRemoteKey { key, at, takes } => write!(
                f,
                "unknown key {key} {}. A remote takes {}; its credentials belong in secrets.lua",
                PlaceText(at),
                takes.join(", ")
            ),
            Misread::WrongType { key, at, expected } => write!(
                f,
                "{key} must be {} {}",
                expected_text(*expected),
                PlaceText(at)
            ),
            Misread::Missing { key, at } => write!(f, "{key} is missing {}", PlaceText(at)),
            Misread::NotAPath { text, at, why } => {
                write!(f, "path {text:?}: {why:?} {}", PlaceText(at))
            }
            Misread::NotAHelperUrl { url, at } => write!(
                f,
                "url {url:?} must look like <helper>::, the helper being letters, digits, - and _ {}",
                PlaceText(at)
            ),
            Misread::NotADuration { key, text, at } => write!(
                f,
                "{key} {text:?} must be <number><s|m|h> {}",
                PlaceText(at)
            ),
        }
    }
}

pub(super) struct PlaceText<'a>(pub(super) &'a Place);

impl fmt::Display for PlaceText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0.table {
            None => write!(f, "at the top level of {}", self.0.file),
            Some(table) => write!(f, "in {table} of {}", self.0.file),
        }
    }
}

fn expected_text(expected: Expected) -> &'static str {
    match expected {
        Expected::Text => "a string",
        Expected::TrueOrFalse => "true or false",
        Expected::ListOfTexts => "a list of strings",
        Expected::ListOfTables => "a list of tables",
        Expected::Table => "a table",
        Expected::TableOfNamedEntries => "a table of named entries",
        Expected::TextOrFunction => "a string or a function",
        Expected::DateTime => "an RFC 3339 date-time",
        Expected::TableOfWholeNumbers => "a table of whole numbers",
    }
}
