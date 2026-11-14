mod argv;
mod misuse;
mod plain;
mod refusal;
mod source;

use source::read;
pub(crate) use source::{decode, read_stdin};

use std::ffi::OsString;
use std::path::Path;

use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, Parser};
use serde_json::{Map, Value};

use crate::args::{Cli, Command, DayArgs, DayCommand};
use crate::error::CliError;
use crate::output::Format;

const TAKING_FLAGS: [&[&str]; 6] = [
    &["new"],
    &["edit"],
    &["done"],
    &["mv"],
    &["rm"],
    &["token", "create"],
];

pub(crate) struct Refused {
    pub(crate) format: Format,
    pub(crate) error: CliError,
}

pub(crate) fn parse(line: Vec<OsString>) -> Result<Cli, Refused> {
    let given = Cli::command()
        .ignore_errors(true)
        .try_get_matches_from(&line)
        .ok();
    let Some((given, source)) = given.and_then(|given| {
        let source = given.get_one::<std::path::PathBuf>("request")?.clone();
        Some((given, source))
    }) else {
        return plain::parse(line);
    };
    let format = Format::of(given.get_flag("json"), given.get_flag("toon"));
    with_request(line, &given, &source).map_err(|error| Refused { format, error })
}

fn with_request(line: Vec<OsString>, given: &ArgMatches, source: &Path) -> Result<Cli, CliError> {
    let (words, leaf_given) = leaf(given);
    let text = read(source)?;
    if words == ["day", "set"] {
        let mut cli = Cli::try_parse_from(line).map_err(|e| CliError::Usage(e.to_string()))?;
        if let Command::Day(DayArgs {
            command: DayCommand::Set { document },
        }) = &mut cli.command
        {
            *document = Some(text);
        }
        return Ok(cli);
    }
    if !TAKING_FLAGS.contains(&words.as_slice()) {
        return Err(CliError::Usage(
            "--request applies to new, edit, done, mv, rm, token create and day set".into(),
        ));
    }
    let fields: Map<String, Value> =
        decode(&text).map_err(|why| CliError::Usage(format!("--request: {why}")))?;
    let leaf_command = command_at(Cli::command(), &words);
    let takes_meta = words != ["token", "create"];
    refuse_flags_beside(given, leaf_given, &leaf_command, &fields, takes_meta)?;
    let mut full = line;
    full.extend(argv::tokens(&leaf_command, &fields, takes_meta)?);
    let cli = Cli::try_parse_from(full).map_err(|e| {
        let mut built = Cli::command();
        built.build();
        refusal::named(&e, &command_at(built, &words))
    })?;
    match &cli.command {
        Command::Edit(args) if args.editor => Err(CliError::Usage(
            "--request: editor cannot be true; a request cannot open an editor".into(),
        )),
        _ => Ok(cli),
    }
}

fn command_at(root: clap::Command, words: &[&str]) -> clap::Command {
    words.iter().fold(root, |command, word| {
        command
            .find_subcommand(word)
            .expect("a parsed command")
            .clone()
    })
}

fn leaf(given: &ArgMatches) -> (Vec<&str>, &ArgMatches) {
    let mut words = Vec::new();
    let mut matches = given;
    while let Some((word, inner)) = matches.subcommand() {
        words.push(word);
        matches = inner;
    }
    (words, matches)
}

fn refuse_flags_beside(
    given: &ArgMatches,
    leaf_given: &ArgMatches,
    leaf_command: &clap::Command,
    fields: &Map<String, Value>,
    takes_meta: bool,
) -> Result<(), CliError> {
    let on_the_line =
        |matches: &ArgMatches, id: &str| matches.value_source(id) == Some(ValueSource::CommandLine);
    if takes_meta && fields.contains_key("meta") && on_the_line(given, "meta") {
        return Err(twice("meta"));
    }
    match leaf_command
        .get_arguments()
        .map(|arg| arg.get_id().as_str())
        .find(|id| on_the_line(leaf_given, id))
    {
        Some(id) if fields.contains_key(id) => Err(twice(id)),
        Some(id) => Err(CliError::Usage(format!(
            "--request takes only global flags beside it; put {id} in the request"
        ))),
        None => Ok(()),
    }
}

fn twice(key: &str) -> CliError {
    CliError::Usage(format!(
        "{key} is in the request and on the command line; give it once"
    ))
}
