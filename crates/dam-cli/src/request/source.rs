use std::io::Read;
use std::path::Path;

use serde::de::DeserializeOwned;

use crate::error::CliError;

pub(super) fn read(source: &Path) -> Result<String, CliError> {
    if source == Path::new("-") {
        return read_stdin();
    }
    std::fs::read_to_string(source)
        .map_err(|e| CliError::Usage(format!("--request: cannot read {}: {e}", source.display())))
}

pub(crate) fn read_stdin() -> Result<String, CliError> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text)?;
    Ok(text)
}

pub(crate) fn decode<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    if text.trim_start().starts_with('{') {
        serde_json::from_str(text).map_err(|e| e.to_string())
    } else {
        toon_format::decode_default(text).map_err(|e| e.to_string())
    }
}
