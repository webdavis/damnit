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

/// The whole interrupt handler: it stores the cancellation flag and returns,
/// which is all a signal handler may safely do. Every wait loop reads the flag.
extern "C" fn on_interrupt(_: libc::c_int) {
    dam_adapters::request_cancellation();
}

fn main() {
    // SAFETY: `on_interrupt` only stores into a static atomic, which is
    // async-signal-safe, and the handler is installed once before any work.
    unsafe {
        libc::signal(
            libc::SIGINT,
            on_interrupt as *const () as libc::sighandler_t,
        )
    };
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
            eprintln!("dam: {e}");
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
