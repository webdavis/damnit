use crate::support;

use support::sandbox::Sandbox;
use support::small_config::{MARKER, small_config};

fn printing_secrets() -> Sandbox {
    let sb = small_config();
    let secrets = std::fs::read_to_string(sb.secrets_path()).unwrap();
    sb.write_secrets_lua(&format!(
        "print(\"{MARKER}-print\")\nio.write(\"{MARKER}-write\\n\")\n\
         io.stdout:write(\"{MARKER}-stdout\\n\")\nio.stderr:write(\"{MARKER}-stderr\\n\")\n\
         io.stdout:setvbuf(\"no\")\nio.output():write(\"{MARKER}-output\\n\")\n{secrets}"
    ));
    sb
}

#[test]
fn what_secrets_lua_prints_reaches_neither_stream_of_list_or_get() {
    let _guard = support::guard("what_secrets_lua_prints_reaches_neither_stream_of_list_or_get");
    let sb = printing_secrets();
    for line in [
        &["config", "list"][..],
        &["config", "list", "--json"],
        &["config", "list", "--toon"],
        &["config", "get", "owner"],
        &["config", "get", "owner", "--json"],
        &["config", "get", "owner", "--toon"],
    ] {
        let out = sb.output(line);
        assert!(out.status.success(), "{line:?}: {out:?}");
        for stream in [&out.stdout, &out.stderr] {
            let text = String::from_utf8_lossy(stream);
            assert!(!text.contains(MARKER), "{line:?}: {text}");
        }
        if line.contains(&"--json") {
            serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap();
        }
    }
}

#[test]
fn validate_shows_what_secrets_lua_prints_on_standard_error_only() {
    let _guard = support::guard("validate_shows_what_secrets_lua_prints_on_standard_error_only");
    let sb = printing_secrets();
    let out = sb.output(&["config", "validate"]);
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stdout.contains(MARKER), "{stdout}");
    for way in ["print", "write", "stdout", "stderr", "output"] {
        assert!(
            stderr.contains(&format!("{MARKER}-{way}\n")),
            "{way}: {stderr}"
        );
    }
}

#[test]
fn what_a_finalizer_in_secrets_lua_prints_is_kept_off_standard_output() {
    let _guard =
        support::guard("what_a_finalizer_in_secrets_lua_prints_is_kept_off_standard_output");
    let sb = small_config();
    let secrets = std::fs::read_to_string(sb.secrets_path()).unwrap();
    sb.write_secrets_lua(&format!(
        "setmetatable({{}}, {{ __gc = function() print(\"{MARKER}-gc\") end }})\n{secrets}"
    ));
    let listed = sb.output(&["config", "list", "--json"]);
    assert!(listed.status.success(), "{listed:?}");
    serde_json::from_slice::<serde_json::Value>(&listed.stdout).unwrap();
    assert!(!String::from_utf8_lossy(&listed.stderr).contains(MARKER));
    let checked = sb.output(&["config", "validate"]);
    assert!(!String::from_utf8_lossy(&checked.stdout).contains(MARKER));
    assert!(String::from_utf8_lossy(&checked.stderr).contains(&format!("{MARKER}-gc\n")));
}
