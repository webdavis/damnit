mod args;
mod commands;
mod context;
mod error;
mod oids;
mod output;
mod prompt;

use clap::Parser;

use crate::args::{Cli, Command};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Format;

fn main() {
    let interrupts_installed = dam_adapters::catch_interrupts();
    let cli = Cli::parse();
    let format = if cli.json {
        Format::Json
    } else if cli.toon {
        Format::Toon
    } else {
        Format::Human
    };
    // A machine format keeps standard error to the one document a failure
    // writes there, so this reaches the operator and not a parser.
    if !interrupts_installed && format == Format::Human {
        eprintln!("dam: could not install the interrupt handler; Ctrl-C will not cancel cleanly");
    }
    match run(cli, format) {
        Ok(()) => {}
        Err(e) => {
            // An interrupt outranks whatever error the abandoned work reported,
            // so a cancelled run exits 3 wherever it was waiting.
            let e = if dam_adapters::cancellation_requested() {
                CliError::Cancelled
            } else {
                e
            };
            output::print_error(format, &e);
            std::process::exit(e.exit_code());
        }
    }
}

fn run(cli: Cli, format: Format) -> Result<(), CliError> {
    if cli.no_pull && !commands::pulls_a_stale_remote(&cli.command) {
        return Err(CliError::Usage(
            "--no-pull applies to ls, show and agenda, the reads that pull a stale remote".into(),
        ));
    }
    let config_path = cli.config.unwrap_or_else(dam_adapters::default_config_path);
    // The catalogue verbs read config and nothing else, so they answer here,
    // before a store is opened: a store that will not open is a failure for
    // the verbs that read one, and no reason to refuse a client the
    // categories and filters it renders its first screen from.
    match cli.command {
        Command::Category(a) => {
            let config = dam_adapters::load_config(&config_path)?;
            let report = commands::catalogue::run_category(&config, a.command)?;
            output::print(format, &report)
        }
        Command::Filter(a) => {
            let config = dam_adapters::load_config(&config_path)?;
            let report = commands::catalogue::run_filter(&config, a.command)?;
            output::print(format, &report)
        }
        command => {
            let store_path = cli.store.unwrap_or_else(dam_adapters::default_store_path);
            let mut ctx = Context::open(config_path, &store_path, format, cli.no_pull)?;
            let report = commands::dispatch(&mut ctx, command)?;
            output::print(format, &report)
        }
    }
}

#[cfg(test)]
mod testing;
