use dam_application::{Standing, TokenWarning, Tokens};

const SEE_WHY: &str = "run dam doctor to see why";
const SEE_IT: &str = "run dam doctor to see it";

pub(super) fn warnings_naming_only_the_token(tokens: &Tokens) -> Vec<String> {
    let disabled = tokens
        .entries
        .iter()
        .filter_map(|token| match token.standing {
            Standing::Disabled(_) => Some(format!("token {} is disabled; {SEE_WHY}", token.name)),
            Standing::Usable(_) => None,
        });
    tokens.warnings.iter().map(text).chain(disabled).collect()
}

fn text(warning: &TokenWarning) -> String {
    match warning {
        TokenWarning::EveryTokenIgnored(_) => format!("every token is ignored; {SEE_WHY}"),
        TokenWarning::Ignored { name, .. } => format!("token {name} is ignored; {SEE_WHY}"),
        TokenWarning::UnknownPermission { name, .. } => {
            format!("token {name}: an unknown permission is skipped; {SEE_IT}")
        }
        TokenWarning::UnknownChannel { name, .. } => {
            format!("token {name}: an unknown channel is skipped; {SEE_IT}")
        }
    }
}
