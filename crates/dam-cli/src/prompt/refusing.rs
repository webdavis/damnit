use dam_application::Refusal;

use super::Prompt;
use crate::error::CliError;

pub(crate) struct RefusingPrompt;

impl Prompt for RefusingPrompt {
    fn choose(&self, _question: &str, _options: &[&str]) -> Result<usize, CliError> {
        Err(refused())
    }

    fn text(&self, _question: &str) -> Result<String, CliError> {
        Err(refused())
    }

    fn can_ask(&self) -> bool {
        false
    }
}

fn refused() -> CliError {
    Refusal::NeedsAnAnswer.into()
}
