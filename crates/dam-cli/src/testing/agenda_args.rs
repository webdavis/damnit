use clap::Parser;

use crate::args::{AgendaArgs, Cli, Command};

pub(crate) fn agenda_args(words: &[&str]) -> AgendaArgs {
    match Cli::try_parse_from(["dam", "agenda"].iter().chain(words)) {
        Ok(Cli {
            command: Command::Agenda(args),
            ..
        }) => args,
        other => panic!("not an agenda command: {other:?}"),
    }
}
