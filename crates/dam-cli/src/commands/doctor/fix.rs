use dam_adapters::SqliteStore;

use super::machine::Machine;
use crate::args::CheckName;
use crate::error::CliError;

pub(super) fn create_what_is_missing(
    names: &[CheckName],
    machine: &Machine,
) -> Result<Vec<String>, CliError> {
    let covers = |name| names.is_empty() || names.contains(&name);
    let config = machine.config_file;
    let mut created = Vec::new();
    let path = config.config_path().display().to_string();
    let config_made = || {
        config
            .create_if_missing()
            .map_err(|e| CliError::Io(format!("could not create {path}: {e}")))
    };
    if covers(CheckName::Config) && config.ends_in_lua().is_ok() && config_made()? {
        created.push(path.clone());
    }
    if covers(CheckName::Database) && SqliteStore::create_if_missing(machine.store_path)? {
        created.push(machine.store_path.display().to_string());
    }
    Ok(created)
}
