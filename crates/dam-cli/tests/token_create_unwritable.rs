mod support;

use support::sandbox::Sandbox;

const BUILT_IN_CODE: &str = "local tokens = {}\nreturn { schema = 1, token = tokens }\n";
const NAME_WRITTEN_TWICE: &str =
    "return { schema = 1, token = { tool = { value = 'a' }, tool = { value = 'b' } } }\n";
const DOES_NOT_LOAD: &str = "return { schema = 1, token = {\n";

#[test]
fn a_secrets_lua_dam_cannot_edit_is_refused_before_a_token_is_made() {
    let _guard = support::guard("a_secrets_lua_dam_cannot_edit_is_refused_before_a_token_is_made");
    for text in [BUILT_IN_CODE, NAME_WRITTEN_TWICE, DOES_NOT_LOAD] {
        let sb = Sandbox::new();
        sb.write_secrets_lua(text);
        for format in [&[][..], &["--json"]] {
            let mut line = vec!["token", "create", "tool", "--allow", "task:read"];
            line.extend_from_slice(format);
            let out = sb.output(&line);
            assert!(!out.status.success(), "{text}: {out:?}");
            assert!(out.stdout.is_empty(), "{text}: {out:?}");
            assert!(
                !String::from_utf8_lossy(&out.stderr).contains("damk_"),
                "{text}: {out:?}"
            );
            assert_eq!(std::fs::read_to_string(sb.secrets_path()).unwrap(), text);
        }
    }
}
