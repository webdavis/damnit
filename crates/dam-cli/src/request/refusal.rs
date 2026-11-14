use crate::error::CliError;

pub(super) fn named(error: &clap::Error, command: &clap::Command) -> CliError {
    let said = super::misuse::sentence(error);
    let mut shown: Vec<(String, &str)> = command
        .get_arguments()
        .map(|arg| (arg.to_string(), arg.get_id().as_str()))
        .collect();
    shown.sort_by_key(|(spelled, _)| std::cmp::Reverse(spelled.len()));
    let named = shown.iter().fold(said, |said, (spelled, key)| {
        said.replace(&format!("'{spelled}'"), key)
            .replace(spelled.as_str(), key)
    });
    CliError::Usage(format!("--request: {named}"))
}
