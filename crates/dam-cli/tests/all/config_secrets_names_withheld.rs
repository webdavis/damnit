use crate::support;

use support::sandbox::Sandbox;
use support::small_config::small_config;

const NAME: &str = "MARKER_s3cret_name";

fn secrets_naming(body: &str) -> Sandbox {
    let sb = small_config();
    sb.write_secrets_lua(&format!("return {{\n  schema = 1,\n{body}\n}}\n"));
    sb
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
        &["config", "list", "-o", "json"],
        &["config", "get", "token.tool"],
        &["config", "get", "token.tool", "-o", "toon"],
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
