use crate::ports::FunctionFailure;

#[derive(Debug, PartialEq, Eq)]
pub enum FilterError {
    Failed {
        filter: String,
        failure: FunctionFailure,
    },
    Withheld {
        filter: String,
    },
}
