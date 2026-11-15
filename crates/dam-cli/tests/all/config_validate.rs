use crate::support;

use support::doctor::listing;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::token_table::TokenTable;

const DOCTOR_CONFIG: &[&str] = &["doctor", "config"];

fn answers_as_doctor_config_does(sb: &Sandbox) {
    for format in [&[][..], &["--json"], &["--toon"]] {
        let validate = sb.output(&[&["config", "validate"][..], format].concat());
        let doctor = sb.output(&[DOCTOR_CONFIG, format].concat());
        assert_eq!(validate.status.code(), doctor.status.code(), "{format:?}");
        assert!(validate.stderr.is_empty(), "{validate:?}");
        assert_eq!(
            String::from_utf8_lossy(&validate.stdout).replace("config validate", "doctor"),
            String::from_utf8_lossy(&doctor.stdout),
            "{format:?}"
        );
    }
    let out = sb.output(&["config", "validate", "--json"]);
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "config-validate.schema.json");
    assert_eq!(answer["command"], "config validate");
}

#[test]
fn good_files_answer_as_doctor_config_does() {
    let _guard = support::guard("good_files_answer_as_doctor_config_does");
    let sb = Sandbox::new();
    sb.add_token("tool", TokenTable::valued("not-a-token"));
    answers_as_doctor_config_does(&sb);
    assert!(sb.output(&["config", "validate"]).status.success());
}

#[test]
fn a_missing_secrets_lua_answers_as_doctor_config_does() {
    let _guard = support::guard("a_missing_secrets_lua_answers_as_doctor_config_does");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.secrets_path()).unwrap();
    answers_as_doctor_config_does(&sb);
    let (ok, out, _) = sb.dam(&["config", "validate"]);
    assert!(ok);
    assert!(
        out.contains("\n  ✓ config: config.lua is valid, no secrets.lua\n"),
        "{out}"
    );
}

#[test]
fn a_broken_config_lua_answers_as_doctor_config_does() {
    let _guard = support::guard("a_broken_config_lua_answers_as_doctor_config_does");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n  owner = ,\n}\n");
    answers_as_doctor_config_does(&sb);
    let out = sb.output(&["config", "validate"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}

#[test]
fn neither_validate_nor_doctor_creates_a_missing_config_lua() {
    let _guard = support::guard("neither_validate_nor_doctor_creates_a_missing_config_lua");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("fresh").join("config.lua");
    let before = listing(sb.dir.path());
    for args in [&["config", "validate"][..], &["doctor"]] {
        let out = sb
            .command(args)
            .env("DAM_CONFIG", &config)
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
        assert!(out.stderr.is_empty(), "{out:?}");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(
            text.contains("  ✓ config: no config.lua, defaults in use\n"),
            "{text}"
        );
        assert_eq!(listing(sb.dir.path()), before, "{args:?}");
    }
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
    answers_as_doctor_config_does(&sb);
    let out = sb.output(&["config", "validate"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("secrets.lua:2: bad token MARKER-s3cret"),
        "{out:?}"
    );
}
