mod support;

use support::sandbox::Sandbox;
use support::small_config::{MARKER, small_config};

fn token_value(sb: &Sandbox) -> String {
    let secrets = std::fs::read_to_string(sb.secrets_path()).unwrap();
    let start = secrets.find("damk_").unwrap();
    let rest = &secrets[start..];
    rest[..rest.find('"').unwrap()].to_string()
}

#[test]
fn no_secret_reaches_any_stream() {
    let _guard = support::guard("no_secret_reaches_any_stream");
    let sb = small_config();
    let token = token_value(&sb);
    for line in [
        &["config", "list"][..],
        &["config", "list", "--json"],
        &["config", "list", "--toon"],
        &["config", "get", "remote.todoist"],
        &["config", "get", "remote.todoist", "--json"],
        &["config", "get", "remote.todoist.credential", "--json"],
        &["config", "get", "token.tool", "--json"],
    ] {
        let out = sb.output(line);
        assert!(out.status.success(), "{line:?}: {out:?}");
        for stream in [&out.stdout, &out.stderr] {
            let text = String::from_utf8_lossy(stream);
            assert!(!text.contains(MARKER), "{line:?}: {text}");
            assert!(!text.contains(&token), "{line:?}: {text}");
        }
    }
}

#[test]
fn a_credential_function_is_named_and_never_called() {
    let _guard = support::guard("a_credential_function_is_named_and_never_called");
    let sb = Sandbox::new();
    let called = sb.dir.path().join("called");
    sb.write_secrets_lua(&format!(
        "return {{\n  schema = 1,\n  remote = {{ fake = {{ api_token = function()\n    io.open({:?}, 'w'):close()\n    error('{MARKER}')\n  end }} }},\n}}\n",
        called.display().to_string()
    ));
    let (ok, out, err) = sb.dam(&["config", "list"]);
    assert!(ok, "{err}");
    assert!(
        out.contains("  remote.fake.credential.api_token  function\n"),
        "{out}"
    );
    assert!(!out.contains(MARKER) && !err.contains(MARKER), "{out}{err}");
    assert!(!called.exists());
}

#[test]
fn what_a_raising_secrets_lua_says_is_never_quoted() {
    let _guard = support::guard("what_a_raising_secrets_lua_says_is_never_quoted");
    let sb = Sandbox::new();
    sb.write_secrets_lua(&format!(
        "local t = \"{MARKER}\"\nerror(\"bad token \" .. t)\n"
    ));
    for (line, exit) in [
        (&["config", "list"][..], 1),
        (&["config", "list", "--json"], 1),
        (&["config", "list", "--toon"], 1),
        (&["config", "print-default"], 0),
        (&["config", "print-default", "--json"], 0),
        (&["config", "print-default", "--toon"], 0),
        (&["config", "get", "owner"], 1),
        (&["config", "get", "owner", "--json"], 1),
        (&["config", "get", "owner", "--toon"], 1),
    ] {
        let out = sb.output(line);
        assert_eq!(out.status.code(), Some(exit), "{line:?}: {out:?}");
        for stream in [&out.stdout, &out.stderr] {
            let text = String::from_utf8_lossy(stream);
            assert!(!text.contains(MARKER), "{line:?}: {text}");
        }
    }
}
