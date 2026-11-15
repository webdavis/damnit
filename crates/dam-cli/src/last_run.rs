use crate::args::{Command, SyncArgs, SyncCommand};
use crate::error::CliError;
use crate::output::{self, Format, Report};

pub(crate) fn asked(command: &Command) -> bool {
    matches!(
        command,
        Command::Sync(SyncArgs {
            command: Some(SyncCommand::Run {
                record_last_run: true
            }),
        })
    )
}

pub(crate) fn recording(
    asked: bool,
    format: Format,
    work: impl FnOnce() -> Result<(), CliError>,
) -> Result<(), CliError> {
    let result = work();
    if let (true, Err(error)) = (asked, &result)
        && let Some(text) = output::error_text(format, error)
        && let Err(unrecorded) = record(text)
    {
        output::print_error(format, &unrecorded);
    }
    result
}

pub(crate) fn record_report(format: Format, report: &Report) -> Result<(), CliError> {
    record(output::render(format, report)?)
}

fn record(mut text: String) -> Result<(), CliError> {
    if !text.ends_with('\n') {
        text.push('\n');
    }
    let path = dam_adapters::default_last_run_path();
    dam_adapters::replace_last_run(&path, &text)
        .map_err(|e| CliError::Io(format!("could not write {}: {e}", path.display())))
}
