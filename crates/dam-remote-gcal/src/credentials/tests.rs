use super::*;

use std::collections::HashMap;

fn environment(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let held: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |name| held.get(name).cloned()
}

#[test]
fn credentials_are_read_under_the_remotes_own_name() {
    let lookup = environment(&[
        ("DAM_WORK_CAL_CLIENT_ID", "id-1"),
        ("DAM_WORK_CAL_CLIENT_SECRET", "GOCSPX-S"),
        ("DAM_WORK_CAL_REFRESH_TOKEN", "1//0gR"),
    ]);
    let c = Credentials::from_lookup("work-cal", lookup).unwrap();
    assert_eq!(c.client.id, "id-1");
    assert_eq!(c.client.secret.expose(), "GOCSPX-S");
    assert_eq!(c.refresh_token.expose(), "1//0gR");
}

#[test]
fn a_missing_credential_names_its_variable_and_its_config_key() {
    let lookup = environment(&[
        ("DAM_HALF_CLIENT_ID", "id-1"),
        ("DAM_HALF_CLIENT_SECRET", "GOCSPX-S"),
        ("DAM_HALF_REFRESH_TOKEN", ""),
    ]);
    let said = Credentials::from_lookup("half", lookup).unwrap_err();
    assert!(said.contains("DAM_HALF_REFRESH_TOKEN"), "{said}");
    assert!(said.contains("refresh_token under [remote.half]"), "{said}");
    assert!(!said.contains("GOCSPX-S"), "{said}");
}
