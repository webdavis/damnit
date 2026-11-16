use super::table::{FilterTable, TableValue};

const FILTER: &str = "filter";
const DEPTH: usize = 16;

type Lookup<'a> = &'a dyn Fn(&str) -> Option<FilterTable>;

impl FilterTable {
    pub fn with_saved(&self, saved: Lookup) -> FilterTable {
        lent(self, saved, DEPTH)
    }
}

fn lent(table: &FilterTable, saved: Lookup, depth: usize) -> FilterTable {
    let mut view = table.clone();
    if depth == 0 {
        return view;
    }
    let texts = table.fields.get(FILTER).map(|v| v.texts().to_vec());
    for text in texts.into_iter().flatten() {
        let names = match text.strip_prefix("not:") {
            Some(_) => continue,
            None => text.strip_prefix('=').map_or_else(
                || text.split(',').map(String::from).collect(),
                |exact| vec![exact.to_string()],
            ),
        };
        let found: Vec<FilterTable> = names
            .iter()
            .filter_map(|name| saved(name))
            .map(|t| lent(&t, saved, depth - 1))
            .collect();
        match found.as_slice() {
            [one] if names.len() == 1 => merge(&mut view, one),
            _ => view.any.extend(found),
        }
    }
    view
}

fn merge(view: &mut FilterTable, lent: &FilterTable) {
    for (key, value) in &lent.fields {
        let joined = match view.fields.remove(key) {
            None => value.clone(),
            Some(held) => {
                let mut texts = held.texts().to_vec();
                texts.extend(value.texts().iter().cloned());
                TableValue::Many(texts)
            }
        };
        view.fields.insert(key.clone(), joined);
    }
    view.any.extend(lent.any.iter().cloned());
}
