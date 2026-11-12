mod support;

use serde_json::Value;
use support::help_flags::own_flags;
use support::sandbox::Sandbox;

#[test]
fn every_token_create_flag_is_named_after_a_key_that_it_prints() {
    let _guard = support::guard("every_token_create_flag_is_named_after_a_key_that_it_prints");
    let sb = Sandbox::new();
    let (ok, out, err) = sb.dam(&["token", "create", "vpt", "--allow", "task:read", "--json"]);
    assert!(ok, "{err}");
    let token: Value = serde_json::from_str(&out).unwrap();
    let (ok, toon, err) = sb.dam(&["token", "create", "vpu", "--allow", "task:read", "--toon"]);
    assert!(ok, "{err}");
    for flag in own_flags(&sb, &["token", "create"]) {
        assert!(
            token
                .get(&flag)
                .or_else(|| token["grants"][0].get(&flag))
                .is_some(),
            "--{flag} names no key in token create --json: {token}"
        );
        assert!(
            toon.contains(&format!("{flag}:")) || toon.contains(&format!("{flag}[")),
            "--{flag} names no key in token create --toon: {toon}"
        );
    }
}
