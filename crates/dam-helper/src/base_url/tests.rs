use super::*;

const VARIABLE: &str = "DAM_SOME_BASE_URL";

#[test]
fn the_base_url_seam_accepts_a_loopback_address_and_nothing_else() {
    assert_eq!(
        loopback_base(VARIABLE, "http://127.0.0.1:8080").unwrap(),
        "http://127.0.0.1:8080"
    );
    assert_eq!(
        loopback_base(VARIABLE, "http://localhost:8080/").unwrap(),
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
        let err = loopback_base(VARIABLE, refused).unwrap_err();
        assert_eq!(
            err,
            format!(
                "DAM_SOME_BASE_URL is a test seam and must be http://127.0.0.1:<port> or \
                 http://localhost:<port>, not {refused:?}"
            )
        );
    }
}

#[test]
fn a_base_url_that_is_not_unicode_is_refused_rather_than_read_as_unset() {
    let not_unicode = VarError::NotUnicode(std::ffi::OsString::from("x"));
    assert_eq!(
        read_base(VARIABLE, Err(not_unicode)).unwrap_err(),
        "DAM_SOME_BASE_URL is a test seam and must be http://127.0.0.1:<port> or \
         http://localhost:<port>; the value it holds is not Unicode"
    );
    assert_eq!(read_base(VARIABLE, Err(VarError::NotPresent)), Ok(None));
    assert_eq!(
        read_base(VARIABLE, Ok("http://127.0.0.1:9/".into())),
        Ok(Some("http://127.0.0.1:9".to_string()))
    );
}
