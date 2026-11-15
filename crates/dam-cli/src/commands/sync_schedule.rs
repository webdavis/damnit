mod view;

use std::ffi::OsString;
use std::path::Path;

use dam_application::{ConfigFile, ScheduledRun};

use crate::args::{Command, SyncArgs, SyncCommand};
use crate::error::CliError;
use crate::output::{self, Format};
use crate::view::report;

use view::{Disabled, Enabled};

pub(crate) enum Change {
    Enable,
    Disable,
}

pub(crate) struct InUse<'a> {
    config: &'a Path,
    store: &'a Path,
}

impl<'a> InUse<'a> {
    pub(crate) fn of(config: &'a Path, store: &'a Path) -> InUse<'a> {
        InUse { config, store }
    }
}

pub(crate) fn asked(command: &Command) -> Option<Change> {
    match command {
        Command::Sync(SyncArgs {
            command: Some(SyncCommand::Enable),
        }) => Some(Change::Enable),
        Command::Sync(SyncArgs {
            command: Some(SyncCommand::Disable),
        }) => Some(Change::Disable),
        _ => None,
    }
}

pub(crate) fn print(
    change: Change,
    config_file: &dyn ConfigFile,
    in_use: InUse,
    format: Format,
    color: bool,
) -> Result<(), CliError> {
    let schedule = dam_adapters::this_systems_schedule();
    let report = match change {
        Change::Enable => {
            let config = config_file.load()?.config;
            let run = scheduled_run(in_use)?;
            let last_run = text(dam_adapters::default_last_run_path().into())?;
            report(&Enabled::new(schedule.install(&run)?, &last_run, &config))?
        }
        Change::Disable => report(&Disabled::new(schedule.remove()?))?,
    };
    output::print(format, color, &report)
}

fn scheduled_run(in_use: InUse) -> Result<ScheduledRun, CliError> {
    Ok(ScheduledRun {
        dam: text(std::env::current_exe()?.into())?,
        config: text(std::path::absolute(in_use.config)?.into())?,
        store: text(std::path::absolute(in_use.store)?.into())?,
        state_home: text(dam_adapters::default_state_home().into())?,
        search_path: text(std::env::var_os("PATH").unwrap_or_default())?,
    })
}

fn text(os: OsString) -> Result<String, CliError> {
    os.into_string().map_err(|os| {
        let shown = Path::new(&os).display();
        CliError::Io(format!(
            "{shown} is not UTF-8, so the schedule cannot name it"
        ))
    })
}
