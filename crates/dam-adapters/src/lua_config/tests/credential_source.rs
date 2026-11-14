use dam_application::{ComputeFailure, CredentialError, CredentialSpec, RemoteName};

use super::support::{Folder, ONE_REMOTE, config, marking_credential, secrets};

fn api_token() -> CredentialSpec {
    CredentialSpec::Computed {
        remote: RemoteName("todoist".into()),
        name: "api_token".into(),
    }
}

fn credential(function: &str) -> String {
    format!(
        "return {{\n  remote = {{\n    todoist = {{\n      api_token = {function},\n    }},\n  }},\n}}\n"
    )
}

fn computed(folder: &Folder) -> Result<String, CredentialError> {
    folder
        .loaded()
        .credentials
        .resolve(&api_token())
        .map(|secret| secret.expose().to_string())
}

fn failure(function: &str) -> ComputeFailure {
    let folder = Folder::under_a_long_folder_name()
        .with_config(ONE_REMOTE)
        .with_secrets(&credential(function));
    match computed(&folder).unwrap_err() {
        CredentialError::ComputeFailed {
            remote,
            name,
            at,
            why,
        } => {
            assert_eq!(
                (remote, name.as_str(), at),
                (
                    RemoteName("todoist".into()),
                    "api_token",
                    format!("{}:4", folder.path("secrets.lua").display())
                )
            );
            why
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_literal_credential_is_returned_as_is() {
    let loaded = secrets("return { remote = { todoist = { api_token = 'abc' } } }").loaded();
    let spec = &loaded.config.remote("todoist").unwrap().credentials[0];
    assert_eq!(loaded.credentials.resolve(spec).unwrap().expose(), "abc");
}

#[test]
fn a_function_credential_is_called_when_its_remote_is_used() {
    let folder = secrets("");
    let marker = folder.marker();
    let folder = folder.with_secrets(&marking_credential(&marker));
    let loaded = folder.loaded();
    assert!(!marker.exists());
    assert_eq!(
        loaded.credentials.resolve(&api_token()).unwrap().expose(),
        "abc"
    );
    assert!(marker.exists(), "the function never ran");
}

#[test]
fn each_load_answers_with_the_functions_it_read() {
    let folder = secrets(&credential("function() return 'first' end"));
    let first = computed(&folder);
    let folder = folder.with_secrets(&credential("function() return 'second' end"));
    assert_eq!(computed(&folder).unwrap(), "second");
    assert_eq!(first.unwrap(), "first");
}

#[test]
fn a_function_that_raises_keeps_its_message_apart_from_the_whole_path_and_line() {
    assert_eq!(
        failure("function() error('the vault is locked') end"),
        ComputeFailure::Raised {
            message: Some("the vault is locked".into())
        }
    );
}

#[test]
fn a_function_that_returns_no_string_names_the_kind_and_never_the_value() {
    let returned = |function| match failure(function) {
        ComputeFailure::ReturnedNotAString { returned } => returned,
        other => panic!("{other:?}"),
    };
    assert_eq!(returned("function() return 31337 end"), "an integer");
    assert_eq!(
        returned("function() return os.getenv('DAM_NO_SUCH_VARIABLE_ZZ') end"),
        "nil"
    );
}

#[test]
fn a_function_that_returns_text_that_is_not_utf8_says_so() {
    assert_eq!(
        failure("function() return '\\255' end"),
        ComputeFailure::ReturnedNonUtf8
    );
}

#[test]
fn a_computed_credential_without_secrets_lua_is_missing() {
    let loaded = config(ONE_REMOTE).loaded();
    assert_eq!(
        loaded.credentials.resolve(&api_token()).unwrap_err(),
        CredentialError::Missing("api_token".into())
    );
}
