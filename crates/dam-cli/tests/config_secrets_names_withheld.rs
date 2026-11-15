mod support;

use support::sandbox::Sandbox;
use support::small_config::small_config;

const NAME: &str = "MARKER_s3cret_name";

fn secrets_naming(body: &str) -> Sandbox {
    let sb = small_config();
    sb.write_secrets_lua(&format!("return {{\n  schema = 1,\n{body}\n}}\n"));
    sb
}

fn every_way_to_hold_a_name() -> Vec<Sandbox> {
    vec![
        secrets_naming(&format!("  {NAME} = 1,")),
        secrets_naming(&format!(
            "  remote = {{ {NAME} = {{ api_token = \"x\" }} }},"
        )),
        secrets_naming(&format!("  remote = {{ todoist = {{ {NAME} = {{}} }} }},")),
        {
            let sb = small_config();
            sb.write_secrets_lua(&format!("local x = 1\ngoto {NAME}\nreturn {{}}\n"));
            sb
        },
    ]
}

#[test]
fn a_name_from_secrets_lua_reaches_no_stream_of_list_get_or_print_default() {
    let _guard =
        support::guard("a_name_from_secrets_lua_reaches_no_stream_of_list_get_or_print_default");
    for sb in every_way_to_hold_a_name() {
        let secrets = std::fs::read_to_string(sb.secrets_path()).unwrap();
        for (line, exit) in [
            (&["config", "list"][..], 1),
            (&["config", "list", "--json"], 1),
            (&["config", "list", "--toon"], 1),
            (&["config", "get", "owner"], 1),
            (&["config", "get", "owner", "--json"], 1),
            (&["config", "get", "owner", "--toon"], 1),
            (&["config", "print-default"], 0),
            (&["config", "print-default", "--json"], 0),
        ] {
            let out = sb.output(line);
            assert_eq!(out.status.code(), Some(exit), "{secrets}{line:?}: {out:?}");
            for stream in [&out.stdout, &out.stderr] {
                let text = String::from_utf8_lossy(stream);
                assert!(!text.contains(NAME), "{secrets}{line:?}: {text}");
            }
            if exit == 1 {
                let said = String::from_utf8_lossy(&out.stderr);
                assert!(said.contains("secrets.lua"), "{secrets}{line:?}: {said}");
                assert!(said.contains("run dam doctor"), "{secrets}{line:?}: {said}");
            }
        }
    }
}

#[test]
fn validate_still_names_what_secrets_lua_holds() {
    let _guard = support::guard("validate_still_names_what_secrets_lua_holds");
    for sb in every_way_to_hold_a_name() {
        let out = sb.output(&["config", "validate"]);
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains(NAME),
            "{out:?}"
        );
    }
}

#[test]
fn a_word_a_token_warning_is_about_reaches_no_stream_of_list_or_get() {
    let _guard = support::guard("a_word_a_token_warning_is_about_reaches_no_stream_of_list_or_get");
    let value = support::tokens::token(1).expose();
    let sb = secrets_naming(&format!(
        "  token = {{ tool = {{ value = \"{value}\", via = {{ \"local\", \"{NAME}\" }},\n    \
         grant = {{ {{ allow = {{ \"task:read\", \"{NAME}\" }} }} }} }} }},"
    ));
    let doctor = sb.doctor(&["doctor"]);
    assert!(doctor.out.contains(NAME), "{}", doctor.out);
    for line in [
        &["config", "list"][..],
        &["config", "list", "--json"],
        &["config", "get", "token.tool"],
        &["config", "get", "token.tool", "--toon"],
    ] {
        let out = sb.output(line);
        assert!(out.status.success(), "{line:?}: {out:?}");
        for stream in [&out.stdout, &out.stderr] {
            let text = String::from_utf8_lossy(stream);
            assert!(!text.contains(NAME), "{line:?}: {text}");
        }
    }
    let said = sb.output(&["config", "list"]).stderr;
    assert_eq!(
        String::from_utf8_lossy(&said),
        "warning: token tool: an unknown channel is skipped; run dam doctor to see it\n\
warning: token tool: an unknown permission is skipped; run dam doctor to see it\n"
    );
}
