use clap::{Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches};
use dam_domain::{FilterTable, SelectField, TableValue, select_fields};

mod any;
mod help_line;
pub(crate) mod ordering;
mod place;

pub(crate) use any::flag_of;

const ANY: &str = "any";

pub(crate) const LIST_HELP: &str = "Every flag repeats, and each one must hold. A comma list holds when any of its values does, not: holds when none does, and none holds when the field is empty.

Examples:
  dam list --labels bug --due ..today
  dam list --done --completed-at week
  dam list --kind event --start today
  dam list --path work/ --any labels=urgent --any due=..today";

pub(crate) const AGENDA_HELP: &str =
    "The selection flags narrow the events in the window, as they do for dam list.

Examples:
  dam agenda --to today
  dam agenda --from week+1 --to week+1 --labels work
  dam agenda --response not:declined --to now+4h";

#[derive(Clone, Debug, Default)]
pub(crate) struct SelectArgs {
    pub(crate) typed: FilterTable,
}

impl SelectArgs {
    pub(crate) fn is_empty(&self) -> bool {
        self.typed == FilterTable::default()
    }
}

impl Args for SelectArgs {
    fn augment_args(command: Command) -> Command {
        command
            .args(select_fields().iter().map(|f| field_arg(f)))
            .arg(any::arg())
    }

    fn augment_args_for_update(command: Command) -> Command {
        Self::augment_args(command)
    }
}

impl FromArgMatches for SelectArgs {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let fields = select_fields()
            .iter()
            .filter_map(|f| Some((f.key.to_string(), typed(matches, f.key)?)))
            .collect();
        let any = matches
            .get_many::<String>(ANY)
            .into_iter()
            .flatten()
            .map(|text| any::condition(text))
            .collect();
        Ok(SelectArgs {
            typed: FilterTable { fields, any },
        })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}

fn typed(matches: &ArgMatches, key: &str) -> Option<TableValue> {
    let mut texts: Vec<String> = matches.get_many::<String>(key)?.cloned().collect();
    Some(match texts.len() {
        1 => TableValue::One(texts.remove(0)),
        _ => TableValue::Many(texts),
    })
}

fn field_arg(field: &'static SelectField) -> Arg {
    let arg = Arg::new(field.key)
        .long(flag_of(field.key))
        .action(ArgAction::Append)
        .value_name(help_line::value_name(field))
        .help(help_line::help(field))
        .help_heading(help_line::heading(field));
    match help_line::is_yes_no(field) {
        true => arg.num_args(0..=1).default_missing_value("true"),
        false => arg,
    }
}

#[cfg(test)]
mod place_tests;
#[cfg(test)]
mod tests;
