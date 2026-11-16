use std::path::Path;

use dam_application::{ConfigError, TaskTemplate};
use mlua::Value;
use mlua::chunk::ChunkMode;

use super::super::evaluate::stopped_at_an_error;
use super::super::task_template::template_in;
use super::locked::locked_lua;

pub(in crate::lua_config) fn read_project_template(
    file: &Path,
    text: &str,
) -> Result<TaskTemplate, ConfigError> {
    let refused = || stopped_at_an_error(file);
    let lua = locked_lua().map_err(|_| refused())?;
    let value: Value = lua
        .load(text)
        .set_name("@template")
        .set_mode(ChunkMode::Text)
        .eval()
        .map_err(|_| refused())?;
    let template = template_in(file, &value).map_err(|_| refused())?;
    if holds_a_control_character(&template) {
        return Err(ConfigError::HoldsAControlCharacter {
            file: file.to_path_buf(),
        });
    }
    Ok(template)
}

fn holds_a_control_character(template: &TaskTemplate) -> bool {
    let control = |text: &str| text.chars().any(char::is_control);
    let body = template.body.as_deref().unwrap_or_default();
    let labels = template.labels.iter().flatten().map(String::as_str);
    let texts = [&template.due, &template.deadline]
        .into_iter()
        .flatten()
        .map(String::as_str);
    let path = template.path.as_ref().map(|p| p.as_str());
    body.chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t')
        || labels.chain(texts).chain(path).any(control)
}
