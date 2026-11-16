use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilterTable {
    pub fields: BTreeMap<String, TableValue>,
    pub any: Vec<FilterTable>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableValue {
    One(String),
    Many(Vec<String>),
}

impl TableValue {
    pub fn texts(&self) -> &[String] {
        match self {
            TableValue::One(text) => std::slice::from_ref(text),
            TableValue::Many(texts) => texts,
        }
    }
}
