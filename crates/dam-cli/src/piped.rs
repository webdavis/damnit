use std::io::Read;
use std::path::Path;

use crate::args::{Cli, Command, Piped, SetArgs};
use crate::commands::mv::with_the_path_named;
use crate::error::{CliError, quoted};
use ids::ids_in;
pub(crate) use standard_input::with_ids_from_standard_input;

pub(crate) fn with_piped_ids(
    mut cli: Cli,
    input: &mut dyn Read,
    terminal: bool,
    typed: &[String],
) -> Result<Option<Cli>, CliError> {
    let request_on_standard_input = cli.request.as_deref() == Some(Path::new("-"));
    let Some((oids, set)) = picked(&mut cli.command)? else {
        return Ok(Some(cli));
    };
    if !oids.iter().any(|word| word == "-") {
        return Ok(Some(cli));
    }
    let command = typed_command(typed);
    if oids.len() > 1 {
        let without_ids = typed
            .iter()
            .filter(|word| *word == "-" || !oids.contains(word));
        return Err(usage(format!(
            "give ids or -, not both: dam list | {}",
            typed_command(&without_ids.cloned().collect::<Vec<_>>())
        )));
    }
    if request_on_standard_input {
        return Err(read_once(typed));
    }
    if terminal {
        return Err(usage(format!(
            "- reads ids from standard input, which is a terminal here; pipe them in: dam list | {command}"
        )));
    }
    let mut text = String::new();
    input.read_to_string(&mut text)?;
    let ids = ids_in(&text, &command)?;
    if ids.is_empty() {
        return Ok(None);
    }
    oids.clear();
    set.piped = Some(Box::new(Piped { ids, command }));
    Ok(Some(cli))
}

pub(crate) fn refuse_reading_twice(
    source: &Path,
    leaf_given: &clap::ArgMatches,
    line: &[std::ffi::OsString],
) -> Result<(), CliError> {
    let oids = leaf_given.try_get_many::<String>("oid").ok().flatten();
    match source == Path::new("-") && oids.is_some_and(|mut oids| oids.any(|oid| oid == "-")) {
        true => Err(read_once(&typed(line))),
        false => Ok(()),
    }
}

pub(crate) fn typed(line: &[std::ffi::OsString]) -> Vec<String> {
    line.iter()
        .skip(1)
        .map(|w| w.to_string_lossy().into())
        .collect()
}

fn read_once(typed: &[String]) -> CliError {
    let mut words = typed.to_vec();
    if let Some(at) = words.windows(2).position(|pair| pair == ["--request", "-"]) {
        words[at + 1] = "FILE".into();
    }
    usage(format!(
        "standard input is read once, for --request - or for -, not both; \
write the request to a file: dam list | {}",
        typed_command(&words)
    ))
}

fn picked(command: &mut Command) -> Result<Option<(&mut Vec<String>, &mut SetArgs)>, CliError> {
    Ok(match command {
        Command::Done(a) => Some((&mut a.oid, &mut a.set)),
        Command::Edit(a) => Some((&mut a.oid, &mut a.set)),
        Command::Rm(a) => Some((&mut a.oid, &mut a.set)),
        Command::Mv(a) => {
            *a = with_the_path_named(std::mem::take(a))?;
            Some((&mut a.oid, &mut a.set))
        }
        _ => None,
    })
}

fn typed_command(typed: &[String]) -> String {
    let words = typed.iter().map(|word| match word.as_str() {
        "-" => word.clone(),
        _ => quoted(word),
    });
    std::iter::once("dam".to_string())
        .chain(words)
        .collect::<Vec<_>>()
        .join(" ")
}

fn usage(message: String) -> CliError {
    CliError::Usage(message)
}

mod ids;
mod standard_input;
#[cfg(test)]
mod tests;
