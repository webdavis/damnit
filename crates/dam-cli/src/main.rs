mod args;
mod commands;
mod context;
mod editor;
mod error;
mod layout;
mod output;
mod prompt;
mod view;

use clap::Parser;
use dam_adapters::{Interrupt, TomlConfigFile};
use dam_application::ConfigFile;

use crate::args::{Cli, Command};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Format;

fn main() {
    let interrupt = Interrupt::default();
    let interrupts_installed = dam_adapters::catch_interrupts(&interrupt);
    let cli = Cli::parse();
    let format = if cli.json {
        Format::Json
    } else if cli.toon {
        Format::Toon
    } else {
        Format::Human
    };
    if !interrupts_installed {
        warn_a_human_that_ctrl_c_will_not_cancel_cleanly(format);
    }
    match run(cli, format, &interrupt) {
        Ok(()) => {}
        Err(e) => {
            let e = cancelled_when_interrupted_whatever_the_work_reported(e, &interrupt);
            output::print_error(format, &e);
            std::process::exit(e.exit_code());
        }
    }
}

fn warn_a_human_that_ctrl_c_will_not_cancel_cleanly(format: Format) {
    if format == Format::Human {
        eprintln!("dam: could not install the interrupt handler; Ctrl-C will not cancel cleanly");
    }
}

fn cancelled_when_interrupted_whatever_the_work_reported(
    e: CliError,
    interrupt: &Interrupt,
) -> CliError {
    if interrupt.requested() {
        CliError::Cancelled
    } else {
        e
    }
}

fn run(cli: Cli, format: Format, interrupt: &Interrupt) -> Result<(), CliError> {
    if cli.no_pull && !commands::pulls_a_stale_remote(&cli.command) {
        return Err(CliError::Usage(
            "--no-pull applies to ls, show and agenda, the reads that pull a stale remote".into(),
        ));
    }
    let config_file =
        TomlConfigFile::new(cli.config.unwrap_or_else(dam_adapters::default_config_path));
    match cli.command {
        Command::Category(a) => {
            let config = config_file.load()?;
            let report = commands::catalogue::run_category(&config, a.command)?;
            output::print(format, &report)
        }
        Command::Filter(a) => {
            let config = config_file.load()?;
            let report = commands::catalogue::run_filter(&config, a.command)?;
            output::print(format, &report)
        }
        command => {
            let store_path = cli.store.unwrap_or_else(dam_adapters::default_store_path);
            let ctx = Context::open(
                Box::new(config_file),
                &store_path,
                format,
                cli.no_pull,
                interrupt,
            )?;
            let report = commands::dispatch_a_verb_that_opens_the_store(&ctx, command)?;
            output::print(format, &report)
        }
    }
}

#[cfg(test)]
mod testing;
