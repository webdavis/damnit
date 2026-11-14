use super::lua_text::{string_literal, table_key};

pub(in crate::lua_config) fn remote_entry(name: &str, url: &str) -> String {
    format!("{} = {{ url = {} }}", table_key(name), string_literal(url))
}
