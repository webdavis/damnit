use super::*;
use dam_application::Refusal;
use dam_domain::QueryError;

fn text(error: UseCaseError) -> String {
    UseCaseText(&error).to_string()
}

#[test]
fn a_refusal_reads_as_its_own_text() {
    assert_eq!(
        text(UseCaseError::Refused(Refusal::NothingToCommit)),
        "nothing to commit"
    );
}

#[test]
fn a_helper_on_protocol_zero_is_told_dam_never_supported_it_and_a_newer_one_to_upgrade_dam() {
    let unsupported = |found| {
        text(UseCaseError::Helper(HelperError::UnsupportedProtocol {
            helper: "dam-remote-todoist".into(),
            found,
            supported: 1,
        }))
    };
    assert_eq!(
        unsupported(0),
        "helper dam-remote-todoist speaks protocol 0, which dam never supported"
    );
    assert_eq!(
        unsupported(2),
        "helper dam-remote-todoist speaks protocol version 2; dam speaks 1, so upgrade dam or use an older helper"
    );
}

#[test]
fn a_missing_helper_dam_ships_names_its_install_command_and_another_says_it_is_its_own() {
    let missing = |helper: &str| {
        text(UseCaseError::Helper(HelperError::NotFound {
            helper: helper.into(),
        }))
    };
    assert_eq!(
        missing("dam-remote-todoist"),
        "dam-remote-todoist was not found on PATH; install it with: cargo install --git https://github.com/webdavis/damnit dam-remote-todoist"
    );
    assert_eq!(
        missing("dam-remote-gcal"),
        "dam-remote-gcal was not found on PATH; install it with: cargo install --git https://github.com/webdavis/damnit dam-remote-gcal"
    );
    assert_eq!(
        missing("dam-remote-other"),
        "dam-remote-other was not found on PATH; a remote's helper is installed on its own"
    );
}

#[test]
fn text_that_is_not_an_oid_asks_for_four_hex_characters() {
    assert_eq!(
        text(UseCaseError::NotAnOid("f5".into())),
        "\"f5\" is not an oid; give at least 4 hex characters"
    );
}

#[test]
fn a_query_error_reads_as_the_query_says_it() {
    assert_eq!(
        text(UseCaseError::Query(QueryError::Unclosed)),
        "a ( has no matching )"
    );
}

#[test]
fn colliding_credentials_ask_for_one_to_be_renamed() {
    assert_eq!(
        text(UseCaseError::Helper(HelperError::CollidingCredentials {
            first: "api-token".into(),
            second: "api_token".into(),
            variable: "DAM_TODOIST_API_TOKEN".into(),
        })),
        "credentials api-token and api_token both become DAM_TODOIST_API_TOKEN; rename one of them"
    );
}

#[test]
fn no_credential_failure_reveals_a_value() {
    const TOKEN: &str = "SUPERSECRETTOKEN";
    let errors = [
        UseCaseError::from(Refusal::MissingCredential {
            remote: "todoist".into(),
            name: "api_token".into(),
        }),
        UseCaseError::from(CredentialError::Missing("api_token".into())),
        UseCaseError::from(CredentialError::ComputeFailed {
            remote: dam_application::RemoteName("todoist".into()),
            name: "api_token".into(),
            at: "secrets.lua:4".into(),
            why: dam_application::ComputeFailure::Raised {
                message: Some("the vault is locked".into()),
            },
        }),
    ];
    for error in errors {
        assert!(!text(error).contains(TOKEN));
    }
}
