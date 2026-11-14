use dam_application::{Standing, Tokens};

use crate::error::{TokenFaultText, TokenWarningText};

pub(crate) fn token_warnings(tokens: &Tokens) -> impl Iterator<Item = String> + '_ {
    let disabled = tokens.entries.iter().filter_map(|t| match &t.standing {
        Standing::Disabled(why) => Some(format!(
            "token {} is disabled: {}",
            t.name,
            TokenFaultText(why)
        )),
        Standing::Usable(_) => None,
    });
    tokens
        .warnings
        .iter()
        .map(|warning| TokenWarningText(warning).to_string())
        .chain(disabled)
}
