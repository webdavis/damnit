use dam_application::CredentialSpec;

use super::helpers::{CONFIG_UNREAD, NO_REMOTES};
use super::machine::{Answer, Machine};
use crate::args::CheckName;
use crate::commands::check_report::Row;
use crate::commands::check_report::{Check, Outcome};

pub(super) fn check(machine: &Machine) -> Vec<Check> {
    match machine.answers() {
        None => vec![found(Outcome::NotRun, CONFIG_UNREAD.into())],
        Some([]) => vec![found(Outcome::NotRun, NO_REMOTES.into())],
        Some(answers) => vec![Check::of_rows(
            CheckName::Credentials,
            answers.iter().map(held).collect(),
        )],
    }
}

fn found(result: Outcome, detail: String) -> Check {
    Check::of(CheckName::Credentials, result, detail)
}

fn held(answer: &Answer) -> Row {
    let remote = &answer.remote.name.0;
    let found = |result, detail: String| Row::of(remote, result, detail);
    let Ok(capabilities) = &answer.capabilities else {
        let detail = "not checked, its helper did not answer".to_string();
        return found(Outcome::NotRun, detail);
    };
    if capabilities.credentials.is_empty() {
        return found(Outcome::Ok, "needs no credential".into());
    }
    let named = |wanted: fn(Option<&CredentialSpec>) -> bool| -> String {
        let specs = &answer.remote.credentials;
        let names = capabilities.credentials.iter().map(String::as_str);
        names
            .filter(|name| wanted(specs.iter().find(|spec| spec.name() == *name)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let set_it = |names: &str| format!("set {names} under remote.{remote} in secrets.lua");
    let missing = named(|spec| spec.is_none());
    if !missing.is_empty() {
        let detail = format!("has no {missing}");
        return found(Outcome::Issue, detail).advising(set_it(&missing));
    }
    let empty = named(
        |spec| matches!(spec, Some(CredentialSpec::Literal { value, .. }) if value.expose().is_empty()),
    );
    if !empty.is_empty() {
        let detail = format!("{empty} is empty");
        return found(Outcome::Issue, detail).advising(set_it(&empty));
    }
    let computed = named(|spec| matches!(spec, Some(CredentialSpec::Computed { .. })));
    let detail = match computed.is_empty() {
        true => "has its credential, not tested".to_string(),
        false => format!("{computed} comes from a function dam did not run"),
    };
    found(Outcome::Ok, detail).advising(format!("run dam pull {remote} to try it"))
}
