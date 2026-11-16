use serde_json::Value;

use crate::error::CliError;

pub(super) fn ids_in(text: &str, command: &str) -> Result<Vec<String>, CliError> {
    if text.trim_start().starts_with('{') {
        return ids_in_the_document(text, command);
    }
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(at, line)| match line.split_whitespace().next() {
            Some(word) if looks_like_an_id(word) => Ok(word.to_string()),
            _ => Err(CliError::Usage(format!(
                "line {} of standard input does not start with an id: {:?}; \
pipe in the rows of dam list or its -o json document: dam list | {command}",
                at + 1,
                line.trim()
            ))),
        })
        .collect()
}

fn ids_in_the_document(text: &str, command: &str) -> Result<Vec<String>, CliError> {
    let document: Option<Value> = serde_json::from_str(text).ok();
    let oids = document
        .as_ref()
        .and_then(|d| d["objects"].as_array())
        .and_then(|objects| {
            objects
                .iter()
                .map(|object| {
                    object["oid"]
                        .as_str()
                        .filter(|oid| looks_like_an_id(oid))
                        .map(String::from)
                })
                .collect::<Option<Vec<_>>>()
        });
    oids.ok_or_else(|| {
        CliError::Usage(format!(
            "standard input holds JSON without the objects of dam list -o json; \
pipe that document in: dam list -o json | {command}"
        ))
    })
}

fn looks_like_an_id(word: &str) -> bool {
    word.len() >= 4 && word.chars().all(|c| c.is_ascii_hexdigit())
}
