use crate::support;

use std::os::unix::fs::PermissionsExt;

use support::sandbox::Sandbox;

const HINT: &str = "run dam doctor";

fn said_by(sb: &Sandbox, line: &[&str]) -> String {
    let (ok, out, err) = sb.dam(line);
    assert!(!ok && out.is_empty(), "{line:?}: {err}");
    err
}

fn each_kind_of_config_error() -> Vec<(Sandbox, &'static str, &'static str)> {
    let returns_an_integer = Sandbox::new();
    returns_an_integer.write_secrets_lua("return 5\n");
    let returns_nil = Sandbox::new();
    returns_nil.write_config_lua("return nil\n");
    let wrong_shape = Sandbox::new();
    wrong_shape.write_secrets_lua("return { schema = 1, remote = { fake = { api_token = 5 } } }\n");
    let syntax = Sandbox::new();
    syntax.write_config_lua("return {\n");
    let unreadable = Sandbox::new();
    std::fs::set_permissions(
        unreadable.config_path(),
        std::fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    vec![
        (
            returns_an_integer,
            "must return a table, and it returned an integer",
            "must return a table, and it returned an integer",
        ),
        (
            returns_nil,
            "must return a table, and it returned nil",
            "must return a table, and it returned nil",
        ),
        (
            wrong_shape,
            "secrets.lua: loading stopped at an error",
            "api_token must be a string or a function",
        ),
        (syntax, "config.lua:", "config.lua:"),
        (unreadable, "config.lua", "config.lua"),
    ]
}

#[test]
fn every_config_error_points_to_doctor_except_doctor_s_own() {
    let _guard = support::guard("every_config_error_points_to_doctor_except_doctor_s_own");
    for (sb, why, doctor_why) in each_kind_of_config_error() {
        let err = said_by(&sb, &["list"]);
        assert!(err.contains(why), "{err}");
        assert_eq!(err.matches(HINT).count(), 1, "{err}");
        assert!(err.trim_end().ends_with(HINT), "{err}");

        let doctor = sb.doctor(&["doctor"]);
        assert_eq!(doctor.code, Some(1), "{}", doctor.out);
        assert!(doctor.out.contains(doctor_why), "{}", doctor.out);
        assert!(!doctor.out.contains(HINT), "{}", doctor.out);
    }
}
