use dam_domain::FilterTable;

use crate::args::selection::flag_of;

pub(super) fn command_line(t: &FilterTable, sort: &[String]) -> String {
    let mut words = vec!["dam".to_string(), "list".to_string()];
    for (key, value) in &t.fields {
        for text in value.texts() {
            words.push(format!("--{}", flag_of(key)));
            words.push(quoted(text));
        }
    }
    for (key, value) in t.any.iter().flat_map(|entry| &entry.fields) {
        for text in value.texts() {
            words.push("--any".into());
            words.push(quoted(&format!("{}={text}", flag_of(key))));
        }
    }
    for text in sort {
        words.push("--sort".into());
        words.push(quoted(text));
    }
    words.join(" ")
}

pub(crate) fn quoted(text: &str) -> String {
    let plain = |c: char| c.is_ascii_alphanumeric() || "._:/,=+-@".contains(c);
    match !text.is_empty() && text.chars().all(plain) {
        true => text.to_string(),
        false => format!("'{}'", text.replace('\'', r"'\''")),
    }
}
