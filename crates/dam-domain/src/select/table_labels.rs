use super::table::FilterTable;

const LABELS: &str = "labels";
const CATEGORY: &str = "category";

type Rename<'a> = &'a dyn Fn(&str) -> Option<String>;

impl FilterTable {
    pub fn labels_renamed(&self, new_of: Rename) -> FilterTable {
        let mut moved = self.clone();
        if let Some(value) = moved.fields.get_mut(LABELS) {
            *value = value.map(|text| relabelled(text, new_of, &label));
        }
        if let Some(value) = moved.fields.get_mut(CATEGORY) {
            *value = value.map(|text| relabelled(text, new_of, &grouped));
        }
        moved.any = self.any.iter().map(|a| a.labels_renamed(new_of)).collect();
        moved
    }
}

fn relabelled(text: &str, new_of: Rename, part: &dyn Fn(&str, Rename) -> String) -> String {
    let (not, rest) = match text.strip_prefix("not:") {
        Some(rest) => ("not:", rest),
        None => ("", text),
    };
    let moved = match rest.strip_prefix('=') {
        Some(exact) => format!("={}", new_of(exact).unwrap_or_else(|| exact.to_string())),
        None => {
            let parts: Vec<String> = rest.split(',').map(|p| part(p, new_of)).collect();
            match parts.as_slice() {
                [one] if needs_exact(one) => format!("={one}"),
                _ => parts.join(","),
            }
        }
    };
    format!("{not}{moved}")
}

fn label(text: &str, new_of: Rename) -> String {
    new_of(text).unwrap_or_else(|| text.to_string())
}

fn grouped(text: &str, new_of: Rename) -> String {
    match text.split_once('=') {
        Some((group, l)) => format!("{group}={}", label(l, new_of)),
        None => label(text, new_of),
    }
}

fn needs_exact(text: &str) -> bool {
    text.contains(',') || text.starts_with("not:") || text.starts_with('=') || text == "none"
}
