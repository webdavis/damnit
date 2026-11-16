use dam_domain::FilterTable;

use super::LabelMap;
use crate::config::{Config, FilterConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigPlace {
    Category { name: String, values: Vec<String> },
    Filter { name: String, table: FilterTable },
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
        .filter_map(|filter| filter_place(filter, map));
    categories.chain(filters).collect()
}

fn filter_place(filter: &FilterConfig, map: &LabelMap) -> Option<ConfigPlace> {
    let new_of = |label: &str| map.new_of(label).map(String::from);
    let table = filter.table.labels_renamed(&new_of);
    if table == filter.table {
        return None;
    }
    let name = filter.name.clone();
    Some(match filter.runs_a_function {
        true => ConfigPlace::HandWritten { name },
        false => ConfigPlace::Filter { name, table },
    })
}
