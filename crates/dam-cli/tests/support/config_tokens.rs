use super::sandbox::Sandbox;
use super::tokens::token;

pub struct ConfigTokens {
    pub sb: Sandbox,
    pub values: Vec<String>,
}

pub fn config_tokens() -> ConfigTokens {
    let sb = Sandbox::new();
    let values: Vec<String> = (1..=4).map(|fill| token(fill).expose()).collect();
    sb.write_secrets_lua(&format!(
        r#"return {{
  schema = 1,
  token = {{
    tool = {{
      value = "{}",
      expires = "2027-05-11T00:00:00Z",
      not_before = "2026-01-01T00:00:00Z",
      via = {{ "mcp" }},
      require = {{ "session" }},
      grant = {{
        {{ allow = {{ "task:read" }}, paths = {{ "work/" }} }},
        {{ allow = {{ "task:create" }}, own = true }},
      }},
    }},
    plain = {{ value = "{}", grant = {{ {{ allow = {{ "task:read" }} }} }} }},
    empty = {{ value = "{}" }},
    off = {{ value = "{}", limits = {{ day = 1 }} }},
    stray = {{ value = "not a token" }},
  }},
}}
"#,
        values[0], values[1], values[2], values[3]
    ));
    ConfigTokens { sb, values }
}
