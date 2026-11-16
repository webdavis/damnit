use dam_domain::label_terms;

use super::LabelMap;
use crate::config::{Config, FilterConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigPlace {
    Category { name: String, values: Vec<String> },
    Filter { name: String, query: String },
    HandWritten { name: String },
}

pub fn config_places(config: &Config, map: &LabelMap) -> Vec<ConfigPlace> {
    let categories = config.categories.all().iter().filter_map(|category| {
        if !category.values.iter().any(|v| map.new_of(v).is_some()) {
            return None;
        }
        let mut values: Vec<String> = Vec::new();
        for value in &category.values {
            let value = map.new_of(value).unwrap_or(value).to_string();
            if !values.contains(&value) {
                values.push(value);
            }
        }
        Some(ConfigPlace::Category {
            name: category.name.clone(),
            values,
        })
    });
    let filters = config
        .filters
        .iter()
        .filter_map(|filter| filter_place(config, filter, map));
    categories.chain(filters).collect()
}

fn filter_place(config: &Config, filter: &FilterConfig, map: &LabelMap) -> Option<ConfigPlace> {
    let query = filter.rule.query()?;
    let terms = label_terms(query, &config.categories).ok()?;
    let renamed: Vec<_> = terms
        .into_iter()
        .filter_map(|term| Some((term.at, map.new_of(&term.label)?)))
        .collect();
    if renamed.is_empty() {
        return None;
    }
    let name = filter.name.clone();
    if filter.rule.runs_a_function() {
        return Some(ConfigPlace::HandWritten { name });
    }
    let mut query = query.to_string();
    for (at, new) in renamed.into_iter().rev() {
        query.replace_range(at, &label_term(new));
    }
    Some(ConfigPlace::Filter { name, query })
}

fn label_term(label: &str) -> String {
    if label
        .chars()
        .any(|c| c.is_whitespace() || "&|!()\"\\".contains(c))
    {
        let escaped = label.replace('\\', "\\\\").replace('"', "\\\"");
        format!("@\"{escaped}\"")
    } else {
        format!("@{label}")
    }
}
