use std::path::Path;

use dam_application::{ConfigError, TaskTemplate};

use super::super::strict::StrictMap;
use super::super::task_template::priority;
use super::rule::misshapen;

const NEW_TASK_KEYS: [&str; 4] = ["labels", "priority", "due", "deadline"];

pub(super) fn new_task(file: &Path, root: &StrictMap) -> Result<TaskTemplate, ConfigError> {
    read(root).map_err(misshapen(file, "new_task"))
}

fn read(root: &StrictMap) -> Result<TaskTemplate, ConfigError> {
    let Some(map) = root.map("new_task")? else {
        return Ok(TaskTemplate::default());
    };
    map.only(&NEW_TASK_KEYS)?;
    Ok(TaskTemplate {
        labels: map.strings("labels")?,
        priority: priority(&map)?,
        due: map.string("due")?,
        deadline: map.string("deadline")?,
        ..TaskTemplate::default()
    })
}
