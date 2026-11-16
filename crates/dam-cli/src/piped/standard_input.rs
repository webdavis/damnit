use std::io::IsTerminal;

use super::with_piped_ids;
use crate::args::Cli;
use crate::output::{Format, print_error};

pub(crate) fn with_ids_from_standard_input(cli: Cli, typed: &[String], format: Format) -> Cli {
    let stdin = std::io::stdin();
    match with_piped_ids(cli, &mut stdin.lock(), stdin.is_terminal(), typed) {
        Ok(Some(cli)) => cli,
        Ok(None) => {
            if format != Format::Quiet {
                eprintln!("dam: no ids came in on standard input; nothing changed");
            }
            std::process::exit(0);
        }
        Err(e) => {
            print_error(format, &e);
            std::process::exit(e.exit_code());
        }
    }
}
