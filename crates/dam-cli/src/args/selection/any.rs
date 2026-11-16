use clap::{Arg, ArgAction};
use dam_domain::{FilterTable, TableValue, select_fields};

use super::ANY;

pub(crate) fn flag_of(key: &str) -> String {
    key.replace(['_', '.'], "-")
}

pub(super) fn arg() -> Arg {
    Arg::new(ANY)
        .long(ANY)
        .action(ArgAction::Append)
        .value_name("FIELD=VALUE")
        .help("Hold at least one of the --any conditions, as well as every other flag")
        .help_heading("Which")
}

pub(super) fn condition(text: &str) -> FilterTable {
    let (field, value) = text.split_once('=').unwrap_or((text, ""));
    let key = select_fields()
        .iter()
        .find(|f| f.key == field || flag_of(f.key) == field)
        .map_or_else(|| field.to_string(), |f| f.key.to_string());
    FilterTable {
        fields: [(key, TableValue::One(value.into()))].into(),
        any: Vec::new(),
    }
}
