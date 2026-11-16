use dam_domain::{DefaultsError, FilterError};

use crate::errors::UseCaseError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectError {
    Value(FilterError),
    Defaults(DefaultsError),
    Sort(FilterError),
}

impl From<SelectError> for UseCaseError {
    fn from(e: SelectError) -> UseCaseError {
        UseCaseError::Select(e)
    }
}
