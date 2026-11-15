mod support;

use support::config_files::Remote;
use support::doctor::{ALL_WELL, listing};
use support::sandbox::Sandbox;
use support::tested::{MARKER, tested};

const SIGNED_IN: &str = "    ✓ fake: signed in";

#[test]
fn a_login_that_works_is_signed_in() {
    let _guard = support::guard("a_login_that_works_is_signed_in");
    let sb = Sandbox::new();
    let answer = sb.test_plain(&["test"]);
    assert_eq!(answer.code, Some(0));
    let rows = ["  ✓ credentials: 1 remote: 1 ok", SIGNED_IN];
    assert_eq!(answer.out, tested(&rows, ALL_WELL));
    assert_eq!(sb.test_plain(&["test", "credentials"]).out, answer.out);
}

#[test]
fn a_refused_credential_shows_what_the_remote_said_and_never_the_credential() {
    let _guard =
        support::guard("a_refused_credential_shows_what_the_remote_said_and_never_the_credential");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::at("fake::").api_token(MARKER));
    let row = "credentials for fake: did not sign in: \
               fake answered 401: token <api_token> is not valid";
    let answer = sb.test_plain(&["test"]);
    assert_eq!(answer.code, Some(1));
    assert_eq!(
        answer.out,
        tested(
            &[
                "  ✗ credentials: 1 remote: 1 issue",
                "    ✗ fake: did not sign in: fake answered 401: token <api_token> is not valid",
                "      → check its credentials in secrets.lua",
            ],
            &["  ✗ 1 issue to fix", &format!("  1. {row}")]
        )
    );
    let (code, document) = sb.test_json(&["test"]);
    assert_eq!(code, Some(1));
    assert_eq!(document["checks"][0]["rows"][0]["result"], "issue");
    for format in [&[][..], &["--json"], &["--toon"]] {
        let out = sb.output(&[&["test"][..], format].concat());
        let said = format!("{out:?}");
        assert!(!said.contains(MARKER), "{format:?}: {said}");
        assert!(out.stderr.is_empty(), "{format:?}: {said}");
    }
}

#[test]
fn a_missing_credential_is_an_issue_naming_where_it_goes() {
    let _guard = support::guard("a_missing_credential_is_an_issue_naming_where_it_goes");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::at("fake::"));
    let answer = sb.test_plain(&["test"]);
    assert_eq!(answer.code, Some(1));
    let lines: Vec<&str> = answer.out.lines().skip(3).take(3).collect();
    assert_eq!(
        lines,
        [
            "  ✗ credentials: 1 remote: 1 issue",
            "    ✗ fake: has no api_token",
            "      → set api_token under remote.fake in secrets.lua",
        ]
    );
}

#[test]
fn a_credential_function_runs_exactly_once() {
    let _guard = support::guard("a_credential_function_runs_exactly_once");
    let sb = Sandbox::new();
    let calls = sb.dir.path().join("calls");
    sb.write_secrets_lua(&format!(
        "return {{ schema = 1, remote = {{ fake = {{ api_token = function()\n  \
         io.open({:?}, 'a'):write('x')\n  return 'tok'\nend }} }} }}\n",
        calls.display().to_string()
    ));
    let answer = sb.test_plain(&["test"]);
    assert_eq!(answer.code, Some(0), "{}", answer.out);
    assert!(answer.out.contains(SIGNED_IN), "{}", answer.out);
    assert_eq!(std::fs::read_to_string(calls).unwrap(), "x");
}

#[test]
fn a_helper_not_installed_is_an_issue() {
    let _guard = support::guard("a_helper_not_installed_is_an_issue");
    let sb = Sandbox::new();
    sb.add_remote("work", Remote::at("tool::").api_token("tok"));
    let answer = sb.test_plain(&["test"]);
    assert_eq!(answer.code, Some(1));
    assert!(
        answer.out.contains(
            "    ✗ work: dam-remote-tool is not installed\n      \
             → install it, then run dam test credentials\n"
        ),
        "{}",
        answer.out
    );
}

#[test]
fn a_helper_that_cannot_test_a_login_is_a_warning_and_is_never_sent_one() {
    let _guard =
        support::guard("a_helper_that_cannot_test_a_login_is_a_warning_and_is_never_sent_one");
    let sb = Sandbox::new();
    sb.install_helper_without_login("old");
    sb.add_remote("old", Remote::at("old::"));
    let calls = sb.dir.path().join("calls");
    sb.write_secrets_lua(&format!(
        "return {{ schema = 1, remote = {{ fake = {{ api_token = 'tok' }}, \
         old = {{ api_token = function() io.open({:?}, 'a'):write('x') return 'tok' end }} }} }}\n",
        calls.display().to_string()
    ));
    let answer = sb.test_plain(&["test"]);
    assert_eq!(answer.code, Some(0));
    assert!(
        answer.out.contains(
            "    ⚠ old: dam-remote-old cannot test a login\n      → update dam-remote-old\n"
        ),
        "{}",
        answer.out
    );
    assert!(!sb.requests_to("old").contains("login"));
    assert!(!calls.exists());
}

#[test]
fn nothing_is_written_to_the_config_or_store_folders() {
    let _guard = support::guard("nothing_is_written_to_the_config_or_store_folders");
    let sb = Sandbox::new();
    let elsewhere = tempfile::tempdir().unwrap();
    let before = listing(sb.dir.path());
    let out = sb
        .command(&["test"])
        .env("FAKE_TOKEN_FILE", elsewhere.path().join("token.txt"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(listing(sb.dir.path()), before);
}
