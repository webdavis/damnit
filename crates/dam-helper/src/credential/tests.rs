use super::*;

#[test]
fn the_token_is_read_under_the_remotes_own_name() {
    let lookup = |variable: &str| (variable == "DAM_WORK_API_TOKEN").then(|| "tok".to_string());
    assert_eq!(
        credential("work", "api_token", lookup).unwrap().expose(),
        "tok"
    );
    let err = credential("todoist", "api_token", lookup).unwrap_err();
    assert_eq!(
        err,
        "DAM_TODOIST_API_TOKEN is not set; declare api_token under [remote.todoist]"
    );
}

#[test]
fn an_empty_token_is_no_token() {
    let err = credential("work", "api_token", |_| Some(String::new())).unwrap_err();
    assert!(err.starts_with("DAM_WORK_API_TOKEN is not set"), "{err}");
}

#[test]
fn a_missing_credential_names_its_variable_and_its_config_key() {
    let lookup = |variable: &str| (variable == "DAM_HALF_CLIENT_SECRET").then(|| "GOCSPX-S".into());
    let said = credential("half", "refresh_token", lookup).unwrap_err();
    assert_eq!(
        said,
        "DAM_HALF_REFRESH_TOKEN is not set; declare refresh_token under [remote.half]"
    );
}
