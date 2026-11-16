use crate::select::{FilterTable, TableValue};

pub fn one(text: &str) -> TableValue {
    TableValue::One(text.into())
}

pub fn many(texts: &[&str]) -> TableValue {
    TableValue::Many(texts.iter().map(|t| t.to_string()).collect())
}

pub fn table(fields: &[(&str, TableValue)]) -> FilterTable {
    FilterTable {
        fields: fields
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
        any: Vec::new(),
    }
}

pub fn either(mut base: FilterTable, any: Vec<FilterTable>) -> FilterTable {
    base.any = any;
    base
}
