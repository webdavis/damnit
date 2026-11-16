use super::*;
use crate::config::CredentialSpec;
use crate::ports::CredentialError;
use crate::testing::{EchoCredentials, FakeProgress, Said, todoist};

fn remote(specs: Vec<CredentialSpec>) -> RemoteConfig {
    RemoteConfig {
        name: todoist(),
        helper: "todoist".into(),
        url: "todoist::".into(),
        credentials: specs,
        deadline: None,
        sync: Default::default(),
        path: None,
        except: Vec::new(),
    }
}

#[test]
fn each_declared_credential_is_resolved_by_name() {
    let r = remote(vec![CredentialSpec::Literal {
        name: "api_token".into(),
        value: "abc".into(),
    }]);
    let out = resolve_credentials(
        &EchoCredentials,
        &r,
        &["api_token".into()],
        &FakeProgress::default(),
    )
    .unwrap();
    assert_eq!(out, vec![("api_token".into(), "abc".into())]);
}

#[test]
fn a_declared_credential_with_no_spec_is_refused_by_name() {
    let r = remote(vec![]);
    let err = resolve_credentials(
        &EchoCredentials,
        &r,
        &["api_token".into()],
        &FakeProgress::default(),
    )
    .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::MissingCredential {
            remote: "todoist".into(),
            name: "api_token".into()
        })
    );
}

#[test]
fn nothing_on_the_credential_path_reveals_the_value() {
    const TOKEN: &str = "SUPERSECRETTOKEN";
    let r = remote(vec![CredentialSpec::Literal {
        name: "api_token".into(),
        value: TOKEN.into(),
    }]);
    let resolved = resolve_credentials(
        &EchoCredentials,
        &r,
        &["api_token".into()],
        &FakeProgress::default(),
    )
    .unwrap();
    assert!(!format!("{resolved:?}").contains(TOKEN));
    let errors = [
        UseCaseError::from(Refusal::MissingCredential {
            remote: "todoist".into(),
            name: "api_token".into(),
        }),
        UseCaseError::from(CredentialError::Missing("api_token".into())),
    ];
    for e in errors {
        assert!(!format!("{e:?}").contains(TOKEN));
    }
}

#[test]
fn undeclared_specs_are_not_resolved() {
    let r = remote(vec![CredentialSpec::Literal {
        name: "extra".into(),
        value: "x".into(),
    }]);
    assert!(
        resolve_credentials(&EchoCredentials, &r, &[], &FakeProgress::default())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_credential_computed_by_a_function_is_a_step_and_a_literal_is_not() {
    let r = remote(vec![
        CredentialSpec::Literal {
            name: "user".into(),
            value: "ada".into(),
        },
        CredentialSpec::Computed {
            remote: todoist(),
            name: "api_token".into(),
        },
    ]);
    let progress = FakeProgress::default();
    let declared = ["user".into(), "api_token".into()];
    resolve_credentials(&EchoCredentials, &r, &declared, &progress).unwrap();
    assert_eq!(
        *progress.said.borrow(),
        [
            Said::Started(
                "credential@todoist: reading todoist api_token from secrets.lua ".into(),
                None
            ),
            Said::Finished,
        ]
    );
}
