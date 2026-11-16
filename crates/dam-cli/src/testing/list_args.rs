use clap::Parser;

use crate::args::{Cli, Command, ListArgs};

pub(crate) fn list_args(words: &[&str]) -> ListArgs {
    match Cli::try_parse_from(["dam", "list"].iter().chain(words)) {
        Ok(Cli {
            command: Command::List(args),
            ..
        }) => args,
        other => panic!("not a list command: {other:?}"),
    }
}
