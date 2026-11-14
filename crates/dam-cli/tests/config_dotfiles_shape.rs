mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, token};

#[test]
fn the_two_files_a_dotfiles_template_renders_serve_the_owner_and_a_token() {
    let _guard =
        support::guard("the_two_files_a_dotfiles_template_renders_serve_the_owner_and_a_token");
    let sb = Sandbox::new();
    let held = token(1).expose();
    sb.write_config_lua(r#"return { remote = { fake = { url = "fake::" } } }"#);
    sb.write_secrets_lua(&format!(
        r#"return {{ schema = 1, remote = {{ fake = {{ api_token = "tok-123" }} }}, token = {{ tool = {{ value = "{held}", grant = {{ {{ allow = {{ "task:create", "task:change", "task:read" }}, own = true }} }} }} }} }}"#
    ));

    let remotes = answer(
        &sb.output(&["remote", "list", "--json"]),
        "remote-list.schema.json",
    );
    assert_eq!(remotes["remotes"][0]["name"], "fake", "{remotes}");

    let tokens = answer(
        &sb.output(&["token", "list", "--json"]),
        "token-list.schema.json",
    );
    assert_eq!(tokens["tokens"][0]["name"], "tool", "{tokens}");
    assert_eq!(tokens["tokens"][0]["standing"], "active", "{tokens}");

    let sent = answer(
        &sb.output_holding(&held, &["new", "write the report", "--json"]),
        "sent.schema.json",
    );
    assert_eq!(sent["subject"], "write the report", "{sent}");
}
