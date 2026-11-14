use super::*;

#[test]
fn a_secret_never_debug_prints_its_value() {
    let secret = Secret::from("1//0gSUPERSECRETREFRESH");
    assert_eq!(format!("{secret:?}"), "Secret(<redacted>)");
    assert_eq!(secret.expose(), "1//0gSUPERSECRETREFRESH");
}
