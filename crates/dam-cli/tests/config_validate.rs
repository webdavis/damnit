mod support;

use std::process::Output;

use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::token_table::TokenTable;

fn both(sb: &Sandbox, format: &[&str]) -> (Output, Output) {
    (
        sb.output(&[&["config", "validate"][..], format].concat()),
        sb.output(&[&["doctor"][..], format].concat()),
    )
}

fn answers_as_doctor_does(sb: &Sandbox) {
    for format in [&[][..], &["--json"], &["--toon"]] {
        let (validate, doctor) = both(sb, format);
        assert_eq!(validate.status.code(), doctor.status.code(), "{format:?}");
        assert_eq!(validate.stdout, doctor.stdout, "{format:?}");
        assert_eq!(validate.stderr, doctor.stderr, "{format:?}");
    }
}

#[test]
fn good_files_answer_as_doctor_does() {
    let _guard = support::guard("good_files_answer_as_doctor_does");
    let sb = Sandbox::new();
    sb.add_token("tool", TokenTable::valued("not-a-token"));
    answers_as_doctor_does(&sb);
    let out = sb.output(&["config", "validate", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "config-validate.schema.json");
}

#[test]
fn a_missing_secrets_lua_answers_as_doctor_does() {
    let _guard = support::guard("a_missing_secrets_lua_answers_as_doctor_does");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.secrets_path()).unwrap();
    answers_as_doctor_does(&sb);
    let (ok, out, _) = sb.dam(&["config", "validate"]);
    assert!(ok);
    assert!(out.ends_with("\nno secrets.lua\n"), "{out}");
}

#[test]
fn a_broken_config_lua_answers_as_doctor_does() {
    let _guard = support::guard("a_broken_config_lua_answers_as_doctor_does");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n  owner = ,\n}\n");
    answers_as_doctor_does(&sb);
    let out = sb.output(&["config", "validate"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}

#[test]
fn validate_creates_no_missing_config_lua_while_doctor_still_does() {
    let _guard = support::guard("validate_creates_no_missing_config_lua_while_doctor_still_does");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("fresh").join("config.lua");
    let run = |args: &[&str]| {
        sb.command(args)
            .env("DAM_CONFIG", &config)
            .output()
            .unwrap()
    };
    let out = run(&["config", "validate"]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    assert_eq!(out.stdout, b"no config.lua\nno secrets.lua\n");
    assert!(!config.parent().unwrap().exists());
    let out = run(&["doctor"]);
    assert!(out.status.success(), "{out:?}");
    assert!(config.exists());
}

#[test]
fn a_token_is_refused_before_the_config_is_read() {
    let _guard = support::guard("a_token_is_refused_before_the_config_is_read");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    let out = sb.output_holding("damk_v1_whatever", &["config", "validate", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(out.stdout.is_empty());
    assert_eq!(error_document(&out)["error"]["rule"], "not_open_to_tokens");
}

#[test]
fn a_runtime_error_in_secrets_lua_is_quoted_by_validate_as_by_doctor() {
    let _guard =
        support::guard("a_runtime_error_in_secrets_lua_is_quoted_by_validate_as_by_doctor");
    let sb = Sandbox::new();
    sb.write_secrets_lua("local t = \"MARKER-s3cret\"\nerror(\"bad token \" .. t)\n");
    answers_as_doctor_does(&sb);
    let (ok, out, err) = sb.dam(&["config", "validate"]);
    assert!(!ok && out.is_empty());
    assert!(
        err.contains("secrets.lua:2: bad token MARKER-s3cret"),
        "{err}"
    );
}
