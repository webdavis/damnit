mod args;
mod commands;
mod context;
mod error;
mod oids;
mod output;
mod prompt;

use clap::Parser;

use crate::args::Cli;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Format;

fn main() {
    if !dam_adapters::catch_interrupts() {
        eprintln!("dam: could not install the interrupt handler; Ctrl-C will not cancel cleanly");
    }
    let cli = Cli::parse();
    let format = if cli.json {
        Format::Json
    } else if cli.toon {
        Format::Toon
    } else {
        Format::Human
    };
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
    let config_path = cli.config.unwrap_or_else(dam_adapters::default_config_path);
    let store_path = cli.store.unwrap_or_else(dam_adapters::default_store_path);
    let mut ctx = Context::open(config_path, &store_path, format)?;
    let report = commands::dispatch(&mut ctx, cli.command)?;
    output::print(format, &report)
}

#[cfg(test)]
mod testing;
