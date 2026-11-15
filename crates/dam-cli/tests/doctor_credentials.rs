mod support;

use std::os::unix::fs::PermissionsExt;

use support::config_files::Remote;
use support::sandbox::Sandbox;

const TRY: &str = "      → run dam pull fake to try it";

fn rows(sb: &Sandbox) -> (Option<i32>, Vec<String>) {
    let answer = sb.doctor(&["doctor", "credentials"]);
    let rows = answer
        .out
        .lines()
        .skip(3)
        .take_while(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    (answer.code, rows)
}

#[test]
fn a_written_credential_is_present_and_untested() {
    let _guard = support::guard("a_written_credential_is_present_and_untested");
    let sb = Sandbox::new();
    let (code, rows) = rows(&sb);
    assert_eq!(code, Some(0));
    assert_eq!(
        rows,
        [
            "  ✓ credentials: 1 remote: 1 ok",
            "    ✓ fake: has its credential, not tested",
            TRY,
        ]
    );
    let given = std::fs::read_to_string(sb.dir.path().join("token.txt")).unwrap();
    assert_eq!(given, "\n");
}

#[test]
fn a_missing_credential_is_an_issue_naming_where_it_goes() {
    let _guard = support::guard("a_missing_credential_is_an_issue_naming_where_it_goes");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::at("fake::"));
    let (code, rows) = rows(&sb);
    assert_eq!(code, Some(1));
    assert_eq!(
        rows,
        [
            "  ✗ credentials: 1 remote: 1 issue",
            "    ✗ fake: has no api_token",
            "      → set api_token under remote.fake in secrets.lua",
        ]
    );
}

#[test]
fn an_empty_credential_counts_as_missing() {
    let _guard = support::guard("an_empty_credential_counts_as_missing");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::at("fake::").api_token(""));
    let (code, rows) = rows(&sb);
    assert_eq!(code, Some(1));
    assert_eq!(
        rows,
        [
            "  ✗ credentials: 1 remote: 1 issue",
            "    ✗ fake: api_token is empty",
            "      → set api_token under remote.fake in secrets.lua",
        ]
    );
}

#[test]
fn a_credential_function_is_never_called() {
    let _guard = support::guard("a_credential_function_is_never_called");
    let sb = Sandbox::new();
    let called = sb.dir.path().join("called");
    sb.write_secrets_lua(&format!(
        "return {{ schema = 1, remote = {{ fake = {{ api_token = function()\n  \
         io.open({:?}, 'w'):write('x')\n  return 'tok'\nend }} }} }}\n",
        called.display().to_string()
    ));
    let (code, rows) = rows(&sb);
    assert_eq!(code, Some(0));
    assert_eq!(
        rows,
        [
            "  ✓ credentials: 1 remote: 1 ok",
            "    ✓ fake: api_token comes from a function dam did not run",
            TRY,
        ]
    );
    assert!(!called.exists());
}

#[test]
fn a_helper_asking_for_no_credential_needs_none() {
    let _guard = support::guard("a_helper_asking_for_no_credential_needs_none");
    let sb = Sandbox::new();
    let helper = sb.dir.path().join("bin/dam-remote-fake");
    std::fs::write(
        &helper,
        "#!/bin/sh\nread -r line\nprintf '{\"protocol\":1,\"kinds\":[\"task\"],\"fields\":[],\"credentials\":[],\"incremental\":false}\\n'\n",
    )
    .unwrap();
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
    let (code, rows) = rows(&sb);
    assert_eq!(code, Some(0));
    assert_eq!(
        rows,
        [
            "  ✓ credentials: 1 remote: 1 ok",
            "    ✓ fake: needs no credential",
        ]
    );
}

#[test]
fn credentials_wait_on_a_helper_that_answers() {
    let _guard = support::guard("credentials_wait_on_a_helper_that_answers");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.dir.path().join("bin/dam-remote-fake")).unwrap();
    let (code, rows) = rows(&sb);
    assert_eq!(code, Some(0));
    assert_eq!(
        rows,
        [
            "  · credentials: 1 remote: 1 not checked",
            "    · fake: not checked, its helper did not answer",
        ]
    );
}

#[test]
fn credentials_are_not_checked_without_a_config_or_a_remote() {
    let _guard = support::guard("credentials_are_not_checked_without_a_config_or_a_remote");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    assert_eq!(
        rows(&sb).1,
        ["  · credentials: not checked, the config did not read"]
    );
    sb.write_config_lua("return {}\n");
    sb.write_secrets_lua("return { schema = 1 }\n");
    assert_eq!(rows(&sb).1, ["  · credentials: no remotes configured"]);
}
