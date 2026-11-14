use serde::{Deserialize, Serialize};

use crate::MutationResult;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarlyAnswer {
    pub early_answer: MutationResult,
}

#[cfg(test)]
mod tests;
