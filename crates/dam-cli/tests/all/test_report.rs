use crate::support;

use support::config_files::Remote;
use support::doctor::VERSION;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::tested::{MARKER, tested};

const REFUSED: &str = "did not sign in: fake answered 401: token <api_token> is not valid";

fn mixed() -> Sandbox {
    let sb = Sandbox::new();
    sb.add_remote("work", Remote::at("fake::").api_token(MARKER));
    sb.install_helper_without_login("old");
    sb.add_remote("old", Remote::at("old::").api_token("tok"));
    sb
}

#[test]
fn several_remotes_report_one_row_each_with_the_worst_on_the_part() {
    let _guard = support::guard("several_remotes_report_one_row_each_with_the_worst_on_the_part");
    let sb = mixed();
    let answer = sb.test_plain(&["test"]);
    assert_eq!(answer.code, Some(1));
    let refused = format!("    ✗ work: {REFUSED}");
    let first = format!("  1. credentials for work: {REFUSED}");
    assert_eq!(
        answer.out,
        tested(
            &[
                "  ✗ credentials: 3 remotes: 1 issue, 1 warning, 1 ok",
                "    ✓ fake: signed in",
                "    ⚠ old: dam-remote-old cannot test a login",
                "      → update dam-remote-old",
                &refused,
                "      → check its credentials in secrets.lua",
            ],
            &[
                "  ✗ 1 issue to fix, 1 warning to look at",
                &first,
                "  2. credentials for old: dam-remote-old cannot test a login",
            ]
        )
    );
}

#[test]
fn the_json_is_the_doctor_shape_with_command_test() {
    let _guard = support::guard("the_json_is_the_doctor_shape_with_command_test");
    let (code, document) = mixed().test_json(&["test"]);
    assert_eq!(code, Some(1));
    assert_eq!(document["version"], VERSION);
    assert_eq!(document["command"], "test");
    let part = &document["checks"][0];
    assert_eq!(part["name"], "credentials");
    assert_eq!(part["result"], "issue");
    let rows: Vec<(&str, &str)> = part["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["name"].as_str().unwrap(),
                row["result"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [("fake", "ok"), ("old", "warning"), ("work", "issue")]
    );
    assert_eq!(part["rows"][1]["advice"], "update dam-remote-old");
    assert_eq!(document["needs_attention"].as_array().unwrap().len(), 2);
}

#[test]
fn a_name_with_nothing_to_test_is_a_usage_error_naming_the_parts() {
    let _guard = support::guard("a_name_with_nothing_to_test_is_a_usage_error_naming_the_parts");
    let sb = Sandbox::new();
    for name in ["config", "database", "helpers", "pushes", "nope"] {
        let out = sb.output(&["test", name]);
        assert_eq!(out.status.code(), Some(2), "{name}: {out:?}");
        assert!(out.stdout.is_empty(), "{out:?}");
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(
            said.contains(name) && said.contains("credentials"),
            "{said}"
        );
    }
    let out = sb.output(&["test", "config", "--json"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(error_document(&out)["error"]["kind"], "usage");
}

#[test]
fn a_token_holder_is_refused_before_anything_is_read() {
    let _guard = support::guard("a_token_holder_is_refused_before_anything_is_read");
    let sb = Sandbox::new();
    let calls = sb.dir.path().join("calls");
    sb.write_secrets_lua(&format!(
        "io.open({:?}, 'a'):write('x')\nreturn {{ schema = 1 }}\n",
        calls.display().to_string()
    ));
    let out = sb.output_holding("damk_v1_whatever", &["test", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert_eq!(error_document(&out)["error"]["rule"], "not_open_to_tokens");
    assert!(!calls.exists());
}
