use dam_application::{Standing, Tokens};

pub(crate) fn token_warnings(tokens: &Tokens) -> impl Iterator<Item = String> + '_ {
    let disabled = tokens.entries.iter().filter_map(|t| match &t.standing {
        Standing::Disabled(why) => Some(format!("token {} is disabled: {why}", t.name)),
        Standing::Usable(_) => None,
    });
    tokens.warnings.iter().cloned().chain(disabled)
}
