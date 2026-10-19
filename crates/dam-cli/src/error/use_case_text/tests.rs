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
        UseCaseError::from(CredentialError::CommandFailed {
            name: "api_token".into(),
            why: "the vault is locked".into(),
        }),
        UseCaseError::from(CredentialError::EnvUnset {
            name: "api_token".into(),
            var: "DAM_TODOIST_API_TOKEN".into(),
        }),
    ];
    for error in errors {
        assert!(!text(error).contains(TOKEN));
    }
}
