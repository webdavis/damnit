use dam_domain::{FilterTable, TableValue};

use super::lua_text::{list_literal, string_literal, table_key};

pub(in crate::lua_config) fn table_literal(table: &FilterTable) -> String {
    let fields = table.fields.iter().map(|(key, value)| {
        let value = match value {
            TableValue::One(text) => string_literal(text),
            TableValue::Many(texts) => list_literal(texts),
        };
        format!("{} = {value}", table_key(key))
    });
    let any = (!table.any.is_empty()).then(|| {
        let alternatives: Vec<String> = table.any.iter().map(table_literal).collect();
        format!("any = {{ {} }}", alternatives.join(", "))
    });
    let parts: Vec<String> = fields.chain(any).collect();
    match parts.is_empty() {
        true => "{}".into(),
        false => format!("{{ {} }}", parts.join(", ")),
    }
}
