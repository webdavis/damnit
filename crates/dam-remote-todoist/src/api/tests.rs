use super::*;

#[test]
fn the_base_url_seam_accepts_a_loopback_address_and_nothing_else() {
    assert_eq!(
        loopback_test_base("http://127.0.0.1:8080").unwrap(),
        "http://127.0.0.1:8080"
    );
    assert_eq!(
        loopback_test_base("http://localhost:8080/").unwrap(),
        "http://localhost:8080"
    );
    for refused in [
        "https://api.todoist.com/api/v1",
        "http://evil.test:8080",
        "http://127.0.0.1.evil.test:8080",
        "http://user@127.0.0.1:8080",
        "http://localhost:8080@evil.test",
        "http://127.0.0.1",
        "http://127.0.0.1:",
        "https://127.0.0.1:8080",
        "",
    ] {
        assert!(loopback_test_base(refused).is_err(), "{refused}");
    }
}

#[test]
fn only_an_unset_base_url_means_todoist_so_one_that_is_not_unicode_is_refused() {
    let not_unicode = std::env::VarError::NotUnicode(std::ffi::OsString::from("x"));
    let err = production_base_unless_set(Err(not_unicode)).unwrap_err();
    assert!(err.contains("DAM_TODOIST_BASE_URL"), "{err}");
    assert_eq!(
        production_base_unless_set(Err(std::env::VarError::NotPresent)).unwrap(),
        PRODUCTION_BASE
    );
    assert_eq!(
        production_base_unless_set(Ok("http://127.0.0.1:9/".into())).unwrap(),
        "http://127.0.0.1:9"
    );
}

#[test]
fn the_api_never_debug_prints_its_token() {
    let api = TodoistApi::new("http://example.test", "SUPERSECRETTOKEN");
    assert!(!format!("{api:?}").contains("SUPERSECRETTOKEN"));
}

#[test]
fn new_trims_a_trailing_slash_from_the_base() {
    let api = TodoistApi::new("http://example.test/", "tok");
    assert_eq!(api.base, "http://example.test");
}

#[test]
fn the_token_is_read_under_the_remotes_own_name() {
    let lookup = |variable: &str| (variable == "DAM_WORK_API_TOKEN").then(|| "tok".to_string());
    assert!(TodoistApi::from_lookup("work", lookup).is_ok());
    let err = TodoistApi::from_lookup("todoist", lookup).unwrap_err();
    assert_eq!(
        err,
        "DAM_TODOIST_API_TOKEN is not set; declare api_token under [remote.todoist]"
    );
}

#[test]
fn an_empty_token_is_no_token() {
    let err = TodoistApi::from_lookup("work", |_| Some(String::new())).unwrap_err();
    assert!(err.starts_with("DAM_WORK_API_TOKEN is not set"), "{err}");
}
