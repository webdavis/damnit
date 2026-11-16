mod folders;

use std::path::Path;

use dam_application::{ConfigError, Expected, TaskTemplate};
use dam_domain::Priority;
use mlua::{Lua, Value};

use super::evaluate::{ErrorText, evaluated};
use super::shape::described;
use super::strict::StrictMap;
pub use folders::FileSystemTemplates;

const TEMPLATE_KEYS: [&str; 6] = ["body", "labels", "priority", "due", "deadline", "path"];

pub(super) fn read_own_template(file: &Path, text: &str) -> Result<TaskTemplate, ConfigError> {
    let lua = Lua::new();
    let value = evaluated(&lua, file, text, ErrorText::QuotesTheFile)?;
    template_in(file, &value)
}

pub(super) fn template_in(file: &Path, value: &Value) -> Result<TaskTemplate, ConfigError> {
    let Value::Table(table) = value else {
        return Err(ConfigError::NotATable {
            path: file.to_path_buf(),
            returned: described(value),
        });
    };
    let short_name = file.file_name().unwrap_or_default().to_string_lossy();
    let root = StrictMap::root(&short_name, table)?;
    root.only(&TEMPLATE_KEYS)?;
    let path = root
        .string("path")?
        .map(|text| dam_domain::Path::parse(&text))
        .transpose()
        .map_err(|_| ConfigError::NotAPath {
            file: file.to_path_buf(),
        })?;
    Ok(TaskTemplate {
        body: root.string("body")?,
        labels: root.strings("labels")?,
        priority: priority(&root)?,
        due: root.string("due")?,
        deadline: root.string("deadline")?,
        path,
    })
}

pub(super) fn priority(root: &StrictMap) -> Result<Option<Priority>, ConfigError> {
    let Some(value) = root.get("priority") else {
        return Ok(None);
    };
    let Value::Integer(number) = value else {
        return Err(root.wrong("priority", Expected::PriorityOneToFour).into());
    };
    u8::try_from(*number)
        .ok()
        .and_then(|number| Priority::new(number).ok())
        .map(Some)
        .ok_or_else(|| root.wrong("priority", Expected::PriorityOneToFour).into())
}
