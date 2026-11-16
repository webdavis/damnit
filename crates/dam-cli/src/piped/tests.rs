mod ids;
mod refusals;

use clap::Parser;

use super::with_piped_ids;
use crate::args::{Cli, Command, SetArgs};
use crate::error::CliError;

const ROWS: &str = "1111111  p2  2026-09-25  home/  owner  buy oat milk  [errand]\n\n2222222  p4  -  -  owner  call mom\n";

fn piping(words: &[&str], input: &str) -> Result<Option<Cli>, CliError> {
    let typed: Vec<String> = words.iter().map(|w| w.to_string()).collect();
    let cli = Cli::try_parse_from(std::iter::once("dam").chain(words.iter().copied())).unwrap();
    with_piped_ids(cli, &mut input.as_bytes(), false, &typed)
}

fn refusal(words: &[&str], input: &str) -> String {
    piping(words, input).unwrap_err().to_string()
}

fn set_of(cli: &Cli) -> (&[String], &SetArgs) {
    match &cli.command {
        Command::Done(a) => (&a.oid, &a.set),
        Command::Edit(a) => (&a.oid, &a.set),
        Command::Mv(a) => (&a.oid, &a.set),
        Command::Rm(a) => (&a.oid, &a.set),
        _ => panic!("not a change command"),
    }
}

fn piped_ids(words: &[&str], input: &str) -> Vec<String> {
    let cli = piping(words, input).unwrap().unwrap();
    let (oids, set) = set_of(&cli);
    assert!(oids.is_empty());
    set.piped.as_ref().unwrap().ids.clone()
}
