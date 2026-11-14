use super::support::{bare_token, secrets};

const WRITTEN_BY_HAND: &str = "-- kept as written\nreturn {\n  schema = 1,\n  remote = {\n    \
                               todoist = {\n      -- read when the remote is used\n      \
                               api_token = function()\n        \
                               return os.getenv(\"TODOIST_API_TOKEN\")\n      end,\n    },\n  },\n  \
                               token = {\n  },\n}\n-- the end\n";

#[test]
fn adding_a_token_keeps_a_function_credential_and_comments_byte_for_byte() {
    let folder = secrets(WRITTEN_BY_HAND);
    let token = bare_token("tool");
    folder.file().add_token(&token).unwrap();
    let value = token.value.expose();
    assert_eq!(
        folder.text("secrets.lua"),
        WRITTEN_BY_HAND.replace(
            "  token = {\n",
            &format!(
                "  token = {{\n    tool = {{\n      value = \"{value}\",\n      \
                 grant = {{\n        {{ allow = {{ \"task:read\" }} }},\n      }},\n    }},\n"
            )
        )
    );
}

#[test]
fn removing_a_token_keeps_a_function_credential_and_comments_byte_for_byte() {
    let folder = secrets(&WRITTEN_BY_HAND.replace(
        "  token = {\n",
        "  token = {\n    tool = { value = \"x\" }, -- made by dam\n",
    ));
    folder.file().remove_token("tool").unwrap();
    assert_eq!(folder.text("secrets.lua"), WRITTEN_BY_HAND);
}

#[test]
fn a_refusal_while_checking_secrets_lua_never_quotes_its_text() {
    for before in [
        "error(\"sekret-1234\")\nreturn { token = { tool = {} } }\n",
        "return {\n  remote = { todoist = { api_token = \"sekret-1234\" sekret } },\n}\n",
    ] {
        let folder = secrets(before);
        let file = folder.file();
        for why in [
            file.add_token(&bare_token("other")).unwrap_err(),
            file.remove_token("tool").unwrap_err(),
        ] {
            let why = why.to_string();
            assert!(why.contains("secrets.lua"), "{why}");
            assert!(!why.contains("sekret"), "{why}");
        }
        assert_eq!(folder.text("secrets.lua"), before);
    }
}
