use dam_application::{HelperError, UseCaseError};

use super::check::{Check, Outcome};
use super::machine::{Answer, Machine};
use super::row::Row;
use crate::args::CheckName;
use crate::error::UseCaseText;

pub(super) const CONFIG_UNREAD: &str = "not checked, the config did not read";
pub(super) const NO_REMOTES: &str = "no remotes configured";

pub(super) fn check(machine: &Machine) -> Vec<Check> {
    match machine.answers() {
        None => vec![Check::of(
            CheckName::Helpers,
            Outcome::NotRun,
            CONFIG_UNREAD,
        )],
        Some([]) => vec![Check::of(CheckName::Helpers, Outcome::NotRun, NO_REMOTES)],
        Some(answers) => vec![Check::of_rows(
            CheckName::Helpers,
            answers.iter().map(answered).collect(),
        )],
    }
}

fn answered(answer: &Answer) -> Row {
    let remote = &answer.remote;
    let helper = format!("dam-remote-{}", remote.helper);
    let found = |result, detail: String| Row::of(&remote.name.0, result, detail);
    match &answer.capabilities {
        Ok(_) => found(Outcome::Ok, format!("{helper} answered")),
        Err(HelperError::NotFound { .. }) => {
            found(Outcome::Issue, format!("{helper} is not installed"))
                .advising("install it, then run dam doctor")
        }
        Err(error) => found(
            Outcome::Issue,
            UseCaseText(&UseCaseError::Helper(error.clone())).to_string(),
        ),
    }
}
