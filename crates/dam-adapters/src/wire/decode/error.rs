use dam_domain::{OidError, PathError, PriorityError};

#[derive(Debug, PartialEq, Eq)]
pub enum WireError {
    Oid {
        field: &'static str,
        value: String,
        why: OidError,
    },
    Path {
        value: String,
        why: PathError,
    },
    Priority {
        value: String,
        why: PriorityError,
    },
    Date {
        field: &'static str,
        value: String,
    },
    UnknownKind(String),
    UnknownEnum {
        field: &'static str,
        value: String,
    },
}

impl WireError {
    pub fn field(&self) -> &'static str {
        match self {
            WireError::Oid { field, .. } => field,
            WireError::Path { .. } => "path",
            WireError::Priority { .. } => "priority",
            WireError::Date { field, .. } => field,
            WireError::UnknownKind(_) => "kind",
            WireError::UnknownEnum { field, .. } => field,
        }
    }

    fn value(&self) -> &str {
        match self {
            WireError::Oid { value, .. }
            | WireError::Path { value, .. }
            | WireError::Priority { value, .. }
            | WireError::Date { value, .. }
            | WireError::UnknownKind(value)
            | WireError::UnknownEnum { value, .. } => value,
        }
    }
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: cannot read {:?}", self.field(), self.value())
    }
}

impl std::error::Error for WireError {}

pub(super) fn oid<'a>(field: &'static str, text: &'a str) -> impl Fn(OidError) -> WireError + 'a {
    move |why| WireError::Oid {
        field,
        value: text.to_string(),
        why,
    }
}

pub(super) fn date(field: &'static str) -> impl Fn(String) -> WireError {
    move |value| WireError::Date { field, value }
}

pub(super) fn word<T>(
    parse: fn(&str) -> Option<T>,
    field: &'static str,
    text: &str,
) -> Result<T, WireError> {
    parse(text).ok_or_else(|| WireError::UnknownEnum {
        field,
        value: text.to_string(),
    })
}
