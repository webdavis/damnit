use std::path::{Path, PathBuf};

use dam_application::ScheduleError;

pub(super) fn write_when_changed(path: &Path, text: &str) -> Result<bool, ScheduleError> {
    if std::fs::read_to_string(path).is_ok_and(|held| held == text) {
        return Ok(false);
    }
    let failed = |e: std::io::Error| failure("write", path, e);
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(failed)?;
    }
    std::fs::write(path, text).map_err(failed)?;
    Ok(true)
}

pub(super) fn remove_each_present(paths: &[PathBuf]) -> Result<Vec<PathBuf>, ScheduleError> {
    let present: Vec<PathBuf> = paths.iter().filter(|p| p.exists()).cloned().collect();
    for path in &present {
        std::fs::remove_file(path).map_err(|e| failure("remove", path, e))?;
    }
    Ok(present)
}

pub(super) fn ran(
    program: &str,
    verb: &str,
    outcome: Result<(), String>,
) -> Result<(), ScheduleError> {
    outcome.map_err(|why| ScheduleError::Failed(format!("{program} {verb}: {why}")))
}

fn failure(verb: &str, path: &Path, e: std::io::Error) -> ScheduleError {
    ScheduleError::Failed(format!("could not {verb} {}: {e}", path.display()))
}
