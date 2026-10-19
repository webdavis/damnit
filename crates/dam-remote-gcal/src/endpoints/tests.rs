use super::*;

#[test]
fn production_names_googles_three_hosts() {
    let e = Endpoints::production();
    assert_eq!(
        e.authorization,
        "https://accounts.google.com/o/oauth2/v2/auth"
    );
    assert_eq!(e.token, "https://oauth2.googleapis.com/token");
    assert_eq!(e.calendar, "https://www.googleapis.com/calendar/v3");
}

#[test]
fn a_loopback_base_serves_all_three_under_one_port() {
    let e = Endpoints::loopback("http://127.0.0.1:8080/").unwrap();
    assert_eq!(e.authorization, "http://127.0.0.1:8080/o/oauth2/v2/auth");
    assert_eq!(e.token, "http://127.0.0.1:8080/token");
    assert_eq!(e.calendar, "http://127.0.0.1:8080/calendar/v3");
    assert!(Endpoints::loopback("http://localhost:9").is_ok());
}

#[test]
fn the_seam_that_carries_credentials_refuses_every_address_but_loopback() {
    assert_eq!(
        Endpoints::loopback("http://evil.test:8080").unwrap_err(),
        "DAM_GCAL_BASE_URL is a test seam and must be http://127.0.0.1:<port> or \
         http://localhost:<port>, not \"http://evil.test:8080\""
    );
}
