use super::*;

#[test]
fn debug_redacts_the_value_and_expose_returns_it() {
    let secret: Secret = "SUPERSECRETTOKEN".into();
    assert_eq!(format!("{secret:?}"), "Secret(<redacted>)");
    assert_eq!(secret.expose(), "SUPERSECRETTOKEN");
}
