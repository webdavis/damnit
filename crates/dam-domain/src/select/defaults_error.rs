use super::fields::{DONE, KIND};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultsError {
    KindContradicts {
        field: &'static str,
    },
    DoneContradicts {
        field: &'static str,
    },
    BothKinds {
        task_field: &'static str,
        event_field: &'static str,
    },
    EventFieldInHookRun {
        field: &'static str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    Drop(&'static str),
    UseAny,
    UseList,
}

impl DefaultsError {
    pub fn fix(&self) -> Fix {
        match self {
            DefaultsError::KindContradicts { .. } => Fix::Drop(KIND.key),
            DefaultsError::DoneContradicts { .. } => Fix::Drop(DONE.key),
            DefaultsError::BothKinds { .. } => Fix::UseAny,
            DefaultsError::EventFieldInHookRun { .. } => Fix::UseList,
        }
    }
}
