use super::fixture::run;
use super::sandbox::Sandbox;

pub const MARKER: &str = "MARKER-s3cret";

const CONFIG: &str = r#"return {
  owner = "Ada",
  remote = {
    todoist = { url = "todoist::", path = "work" },
  },
  category = {
    effort = { values = { "light", "deep" }, exclusive = true },
  },
  filter = {
    today = { due = "..today" },
  },
}
"#;

pub fn small_config() -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(CONFIG);
    sb.write_secrets_lua(&format!(
        "return {{\n  schema = 1,\n  remote = {{ todoist = {{ api_token = \"{MARKER}\" }} }},\n}}\n"
    ));
    run(
        &sb,
        &[
            "token",
            "create",
            "tool",
            "--allow",
            "task:read",
            "--paths",
            "work",
        ],
    );
    sb
}
