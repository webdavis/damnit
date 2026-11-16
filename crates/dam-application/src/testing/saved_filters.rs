use dam_domain::{FilterTable, TableValue};

use crate::config::FilterConfig;

pub fn fields(pairs: &[(&str, &str)]) -> FilterTable {
    FilterTable {
        fields: pairs
            .iter()
            .map(|(k, v)| (k.to_string(), TableValue::One(v.to_string())))
            .collect(),
        any: Vec::new(),
    }
}

pub fn saved(name: &str, pairs: &[(&str, &str)]) -> FilterConfig {
    FilterConfig {
        name: name.into(),
        table: fields(pairs),
        runs_a_function: false,
    }
}

pub fn saved_function(name: &str, pairs: &[(&str, &str)]) -> FilterConfig {
    FilterConfig {
        runs_a_function: true,
        ..saved(name, pairs)
    }
}
