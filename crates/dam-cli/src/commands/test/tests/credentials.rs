use super::{FAKE, FAKE_TOKEN, Files, Helpers, MARKER, attention_of, json, rows_of};
use crate::args::PartName;

#[test]
fn a_login_that_works_is_signed_in() {
    let files = Files::new(FAKE, FAKE_TOKEN);
    let report = files.report(&Helpers::default(), &[]);
    assert_eq!(
        rows_of(&report),
        ["  ✓ credentials: 1 remote: 1 ok", "    ✓ fake: signed in"]
    );
    assert_eq!(attention_of(&report), ["  ✓ nothing needs attention"]);
    assert_eq!(json(&report)["checks"][0]["result"], "ok");
    let asked = files.report(&Helpers::default(), &[PartName::Credentials]);
    assert_eq!(json(&asked), json(&report));
}

#[test]
fn a_refused_credential_shows_what_the_remote_said_and_never_the_credential() {
    let secrets =
        format!("return {{ schema = 1, remote = {{ fake = {{ api_token = {MARKER:?} }} }} }}\n");
    let files = Files::new(FAKE, &secrets);
    let report = files.report(&Helpers::default(), &[]);
    let said = "did not sign in: fake answered 401: token <api_token> is not valid";
    assert_eq!(
        rows_of(&report),
        [
            "  ✗ credentials: 1 remote: 1 issue".to_string(),
            format!("    ✗ fake: {said}"),
            "      → check its credentials in secrets.lua".to_string(),
        ]
    );
    assert_eq!(
        attention_of(&report),
        [
            "  ✗ 1 issue to fix".to_string(),
            format!("  1. credentials for fake: {said}")
        ]
    );
    assert_eq!(json(&report)["checks"][0]["result"], "issue");
    assert_eq!(json(&report)["checks"][0]["rows"][0]["result"], "issue");
    let rendered = crate::view::report(&report).unwrap();
    assert!(!rendered.human.contains(MARKER), "{}", rendered.human);
    assert!(
        !rendered.data.to_string().contains(MARKER),
        "{}",
        rendered.data
    );
    assert!(rendered.standard_error_lines.is_empty());
}

#[test]
fn a_missing_credential_is_an_issue_naming_where_it_goes() {
    let files = Files::new(FAKE, "return { schema = 1 }\n");
    let helpers = Helpers::default();
    let report = files.report(&helpers, &[]);
    assert_eq!(
        rows_of(&report),
        [
            "  ✗ credentials: 1 remote: 1 issue",
            "    ✗ fake: has no api_token",
            "      → set api_token under remote.fake in secrets.lua",
        ]
    );
    assert!(helpers.logins.borrow().is_empty());
}

#[test]
fn a_credential_function_runs_exactly_once() {
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("calls");
    let secrets = format!(
        "return {{ schema = 1, remote = {{ fake = {{ api_token = function()\n  \
         io.open({:?}, 'a'):write('x')\n  return 'tok'\nend }} }} }}\n",
        calls.display().to_string()
    );
    let files = Files::new(FAKE, &secrets);
    let report = files.report(&Helpers::default(), &[]);
    assert_eq!(rows_of(&report)[1], "    ✓ fake: signed in");
    assert_eq!(std::fs::read_to_string(calls).unwrap(), "x");
}

#[test]
fn a_helper_not_installed_is_an_issue() {
    let config = "return { remote = { work = { url = \"tool::\" } } }\n";
    let secrets = "return { schema = 1, remote = { work = { api_token = \"tok\" } } }\n";
    let report = Files::new(config, secrets).report(&Helpers::default(), &[]);
    assert_eq!(
        rows_of(&report)[1..],
        [
            "    ✗ work: dam-remote-tool is not installed",
            "      → install it, then run dam test credentials",
        ]
    );
    assert_eq!(json(&report)["checks"][0]["result"], "issue");
}

#[test]
fn a_helper_that_cannot_test_a_login_is_a_warning_and_is_never_sent_one() {
    let config = "return { remote = { old = { url = \"old::\" } } }\n";
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("calls");
    let secrets = format!(
        "return {{ schema = 1, remote = {{ old = {{ api_token = function() \
         io.open({:?}, 'a'):write('x') return 'tok' end }} }} }}\n",
        calls.display().to_string()
    );
    let helpers = Helpers::default();
    let report = Files::new(config, &secrets).report(&helpers, &[]);
    assert_eq!(
        rows_of(&report)[1..],
        [
            "    ⚠ old: dam-remote-old cannot test a login",
            "      → update dam-remote-old",
        ]
    );
    assert_eq!(json(&report)["checks"][0]["result"], "warning");
    assert!(helpers.logins.borrow().is_empty());
    assert!(!calls.exists());
}
