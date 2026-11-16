use super::table::FilterTable;
use crate::Path;

const PATH: &str = "path";

impl FilterTable {
    pub fn under(&self, place: &Path) -> FilterTable {
        let mut moved = self.clone();
        if *place != Path::root() {
            move_under(&mut moved, place);
        }
        moved
    }
}

fn move_under(t: &mut FilterTable, place: &Path) {
    if let Some(value) = t.fields.get_mut(PATH) {
        *value = value.map(|text| text_under(text, place));
    }
    t.any.iter_mut().for_each(|a| move_under(a, place));
}

fn text_under(text: &str, place: &Path) -> String {
    let (not, rest) = match text.strip_prefix("not:") {
        Some(rest) => ("not:", rest),
        None => ("", text),
    };
    let moved = match rest.strip_prefix('=') {
        Some(exact) => one_under(exact, place).map(|p| format!("={p}")),
        None => rest
            .split(',')
            .map(|part| one_under(part, place))
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join(",")),
    };
    moved.map_or_else(|| text.to_string(), |moved| format!("{not}{moved}"))
}

fn one_under(text: &str, place: &Path) -> Option<String> {
    Path::parse(text)
        .ok()
        .map(|p| p.under(place).as_str().to_string())
}
