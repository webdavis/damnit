use dam_application::{HelperError, HelperLauncher, RemoteName};

use super::testing::{FAKE_THAT_ECHOES_ITS_TOKEN, install, remote};
use super::*;

#[test]
fn find_locates_the_helper_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(dir.path(), FAKE_THAT_ECHOES_ITS_TOKEN);
    assert!(launcher.find("t").is_some());
    assert!(launcher.find("missing").is_none());
}

#[test]
fn a_non_executable_helper_is_not_found_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("dam-remote-t");
    std::fs::write(&file, FAKE_THAT_ECHOES_ITS_TOKEN).unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    let launcher = ProcessLauncher::with_search_path(dir.path().as_os_str(), Interrupt::default());
    assert!(launcher.find("t").is_none());
    let err = launcher.launch(&remote(), &[]).unwrap_err();
    assert_eq!(
        err,
        HelperError::NotFound {
            helper: "dam-remote-t".into()
        }
    );
}

#[test]
fn a_missing_helper_is_not_found_by_name() {
    let launcher = ProcessLauncher::with_search_path("/nonexistent-dam-dir", Interrupt::default());
    let err = launcher.launch(&remote(), &[]).unwrap_err();
    assert_eq!(
        err,
        HelperError::NotFound {
            helper: "dam-remote-t".into()
        }
    );
}

#[test]
fn credentials_arrive_as_environment_and_calls_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(dir.path(), FAKE_THAT_ECHOES_ITS_TOKEN);
    let mut helper = launcher
        .launch(&remote(), &[("api_token".into(), "tok".into())])
        .unwrap();
    let caps = helper.capabilities().unwrap();
    assert_eq!(caps.kinds, vec![dam_domain::Kind::Task]);
    assert_eq!(
        caps.credentials,
        vec!["api_token".to_string(), "tok".to_string()],
        "the credential reached the child as an environment variable"
    );
    assert!(helper.pull(None).unwrap().objects.is_empty());
    let pushed = helper.push(vec![]).unwrap();
    assert_eq!(
        pushed[0].verdict,
        dam_application::MutationVerdict::Accepted {
            remote_id: Some("r1".into())
        }
    );
}

#[test]
fn an_error_line_is_a_remote_error_and_eof_is_io() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(
        dir.path(),
        "#!/bin/sh\nread -r line; printf '{\"error\":\"nope\"}\\n'\n",
    );
    let mut helper = launcher.launch(&remote(), &[]).unwrap();
    assert_eq!(
        helper.capabilities().unwrap_err(),
        HelperError::Remote("nope".into())
    );
    assert!(matches!(
        helper.capabilities().unwrap_err(),
        HelperError::Io(_)
    ));
}

#[test]
fn two_credential_names_that_become_one_variable_are_refused_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(dir.path(), FAKE_THAT_ECHOES_ITS_TOKEN);
    let err = launcher
        .launch(
            &remote(),
            &[
                ("client-id".into(), "one".into()),
                ("client.id".into(), "two".into()),
            ],
        )
        .unwrap_err();
    assert_eq!(
        err,
        HelperError::CollidingCredentials {
            first: "client-id".into(),
            second: "client.id".into(),
            variable: "DAM_T_CLIENT_ID".into(),
        }
    );
}

#[test]
fn the_helper_is_given_its_remote_name_and_address_and_reads_credentials_under_that_name() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(
        dir.path(),
        "#!/bin/sh\nread -r line\nprintf '{\"protocol\":1,\"kinds\":[\"task\"],\"fields\":[],\"credentials\":[\"%s\",\"%s\",\"%s\",\"%s\"],\"incremental\":false}\\n' \"$#\" \"$1\" \"$2\" \"$DAM_WORK_API_TOKEN\"\n",
    );
    let mut remote = remote();
    remote.name = RemoteName("work".into());
    remote.url = "t::primary,team@x".into();
    let mut helper = launcher
        .launch(&remote, &[("api_token".into(), "tok".into())])
        .unwrap();
    assert_eq!(
        helper.capabilities().unwrap().credentials,
        vec![
            "2".to_string(),
            "work".to_string(),
            "primary,team@x".to_string(),
            "tok".to_string()
        ]
    );
}

#[test]
fn an_empty_address_is_still_passed_so_the_count_never_varies() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(
        dir.path(),
        "#!/bin/sh\nread -r line\nprintf '{\"protocol\":1,\"kinds\":[\"task\"],\"fields\":[],\"credentials\":[\"%s\",\"%s\"],\"incremental\":false}\\n' \"$#\" \"$2\"\n",
    );
    let mut helper = launcher.launch(&remote(), &[]).unwrap();
    assert_eq!(
        helper.capabilities().unwrap().credentials,
        vec!["2".to_string(), String::new()]
    );
}
