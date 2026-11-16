use std::path::Path;

use dam_application::{ConfigError, ProjectRuleFault};
use mlua::{Table, Value};

pub(super) fn needs_no_newer_dam(
    file: &Path,
    table: &Table,
    this_dam: &str,
) -> Result<(), ConfigError> {
    let needs = match table.raw_get::<Value>("minimum_dam") {
        Ok(Value::Nil) => return Ok(()),
        Ok(Value::String(text)) => text.to_string_lossy(),
        _ => return Err(not_a_version(file)),
    };
    let wanted = version(&needs).ok_or_else(|| not_a_version(file))?;
    match version(this_dam).is_some_and(|this| this >= wanted) {
        true => Ok(()),
        false => Err(ConfigError::NeedsANewerDam {
            file: file.to_path_buf(),
            needs,
            this: this_dam.to_string(),
        }),
    }
}

fn version(text: &str) -> Option<[u64; 3]> {
    let mut parts = text.split('.').map(|part| part.parse::<u64>().ok());
    let read = [parts.next()??, parts.next()??, parts.next()??];
    parts.next().is_none().then_some(read)
}

fn not_a_version(file: &Path) -> ConfigError {
    ConfigError::ProjectRule {
        file: file.to_path_buf(),
        key: "minimum_dam",
        fault: ProjectRuleFault::Shape,
    }
}
