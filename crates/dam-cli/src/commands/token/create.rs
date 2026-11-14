use std::collections::BTreeSet;

use dam_application::{ConfigError, ConfigFile, NewToken, Randomness, Refusal, TokenName};
use dam_domain::{Channel, Grant, Path, Permission, PermissionName, Timestamp, Token};
use serde::Serialize;

use super::grant_view::GrantView;
use crate::args::CreateArgs;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct TokenCreated {
    name: String,
    key_id: String,
    value: String,
    expires: Option<String>,
    via: Vec<&'static str>,
    require: Vec<String>,
    grants: Vec<GrantView>,
    warnings: Vec<String>,
}

impl View for TokenCreated {
    fn human(&self) -> Human {
        Human::Lines(vec![self.value.clone()])
    }

    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    fn notices(&self) -> Vec<String> {
        vec![
            format!("created token {}, key id {}", self.name, self.key_id),
            "the secret is shown this once".to_string(),
        ]
    }
}

pub(super) fn run(
    file: &dyn ConfigFile,
    random: &dyn Randomness,
    args: CreateArgs,
) -> Result<Report, CliError> {
    let name = TokenName::parse(&args.name).map_err(CliError::Usage)?;
    file.can_add_token(&name).map_err(|e| match e {
        ConfigError::AlreadyHeld(_) => Refusal::TokenExists(args.name.clone()).into(),
        other => CliError::Config(other),
    })?;
    let token = NewToken {
        name,
        value: Token::generate(&mut |b: &mut [u8]| random.fill(b)),
        expires: args.expires.as_deref().map(instant).transpose()?,
        via: some_unless_empty(args.via)
            .map(|words| words.iter().map(|w| channel(w)).collect())
            .transpose()?,
        require: args.require,
        grant: Grant {
            allow: args
                .allow
                .iter()
                .map(|w| permission(w))
                .collect::<Result<_, _>>()?,
            paths: some_unless_empty(args.paths)
                .map(|texts| texts.iter().map(|p| path(p)).collect())
                .transpose()?,
            labels: some_unless_empty(args.labels).map(BTreeSet::from_iter),
            own: args.own,
            remotes: some_unless_empty(args.remotes),
        },
    };
    let warning = file.add_token(&token)?;
    report(&TokenCreated {
        name: token.name.as_str().to_string(),
        key_id: token.value.key_id().to_string(),
        value: token.value.expose(),
        expires: token.expires.map(|at| at.to_string()),
        via: token
            .via
            .unwrap_or_else(|| BTreeSet::from([Channel::Local]))
            .iter()
            .map(|c| c.as_str())
            .collect(),
        require: token.require,
        grants: vec![GrantView::of(&token.grant)],
        warnings: Vec::from_iter(warning),
    })
}

fn some_unless_empty(values: Vec<String>) -> Option<Vec<String>> {
    (!values.is_empty()).then_some(values)
}

fn permission(word: &str) -> Result<Permission, CliError> {
    match PermissionName::classify(word) {
        PermissionName::Grantable(permission) => Ok(permission),
        PermissionName::Reserved => Err(Refusal::NotGrantable(word.to_string()).into()),
        PermissionName::Unknown => Err(CliError::Usage(format!("unknown permission {word:?}"))),
    }
}

fn channel(word: &str) -> Result<Channel, CliError> {
    Channel::parse(word)
        .ok_or_else(|| CliError::Usage(format!("unknown channel {word:?}; use local or mcp")))
}

fn path(text: &str) -> Result<Path, CliError> {
    Path::parse(text).map_err(|e| CliError::Usage(format!("path {text:?}: {e}")))
}

fn instant(text: &str) -> Result<Timestamp, CliError> {
    text.parse().map_err(|_| {
        CliError::Usage(format!(
            "--expires {text:?} must be a date-time with an offset, such as 2027-05-11T00:00:00Z"
        ))
    })
}

#[cfg(test)]
mod tests;
