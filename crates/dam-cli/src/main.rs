mod args;
mod commands;
mod context;
mod error;
mod oids;
mod output;
mod prompt;
#[cfg(test)]
mod testing;

use clap::Parser;

use crate::args::Cli;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Format;

fn main() {
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
            eprintln!("dam: {e}");
            std::process::exit(e.exit_code());
        }
    }
}

fn run(cli: Cli, format: Format) -> Result<(), CliError> {
    let config_path = cli.config.unwrap_or_else(dam_adapters::default_config_path);
    let store_path = cli.store.unwrap_or_else(dam_adapters::default_store_path);
    let mut ctx = Context::open(config_path, &store_path)?;
    let report = commands::dispatch(&mut ctx, cli.command)?;
    output::print(format, &report)
}
