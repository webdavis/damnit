use dam_application::{Config, Refusal, SelectError, SettingValue, SettingsFile, UseCaseError};
use dam_domain::{
    Command, Filter, FilterTable, NameFault, chain_to_itself, filter_name_fault, scope,
};

use super::entry::entry_key;
use super::filter_written::{FilterCreated, FilterRemoved};
use crate::commands::asked::Asked;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{flags_of, report, table_json};

pub(super) fn create(
    config: &Config,
    file: &dyn SettingsFile,
    named: &str,
    name: &str,
    table: &FilterTable,
) -> Result<Report, CliError> {
    if let Some(fault) = filter_name_fault(name) {
        return Err(CliError::Usage(name_refused(name, fault)));
    }
    if config.filters.iter().any(|f| f.name == name) {
        return Err(CliError::Usage(format!(
            "filter {name} exists; run dam filter remove {name} first, or pick another name"
        )));
    }
    let asked = Asked {
        verb: format!("dam filter create {name}"),
        typed: table,
        sort: &[],
    };
    let filter = asked.filter()?;
    filter
        .check(&config.categories)
        .map_err(|e| asked.fault(SelectError::Value(e)))?;
    scope(&filter, Command::List).map_err(|e| asked.fault(SelectError::Defaults(e)))?;
    refuse_a_chain(config, name, &filter)?;
    file.set(
        &entry_key("filter", name),
        &SettingValue::Table(table.clone()),
    )?;
    let shown = flags_of(table).unwrap_or_else(|| table_json(table).to_string());
    report(&FilterCreated {
        name,
        table: table_json(table),
        shown,
        file: named,
    })
}

fn name_refused(name: &str, fault: NameFault) -> String {
    let why = match fault {
        NameFault::Empty => return "a filter needs a name".into(),
        NameFault::None => format!("a filter cannot be named {name}, which means an empty field"),
        NameFault::OnDay => format!("a filter cannot be named {name}, which on: reads as a day"),
        NameFault::Starts(start) => {
            format!("a filter's name cannot start with {start}, which a value reads")
        }
        NameFault::Comma => {
            "a filter's name cannot hold a comma, which separates names in a list".into()
        }
    };
    format!("{why}; pick another name")
}

fn refuse_a_chain(config: &Config, name: &str, filter: &Filter) -> Result<(), CliError> {
    let reaches = |at: &str| -> Vec<String> {
        let names = match at == name {
            true => filter.saved_filters(),
            false => {
                let held = config.filters.iter().find(|f| f.name == at);
                let read = held.and_then(|f| Filter::read(&f.table).ok());
                return read
                    .map(|f| f.saved_filters().into_iter().map(String::from).collect())
                    .unwrap_or_default();
            }
        };
        names.into_iter().map(String::from).collect()
    };
    match chain_to_itself(name, reaches) {
        Some(chain) => Err(CliError::UseCase(UseCaseError::Refused(
            Refusal::FilterReachesItself(chain),
        ))),
        None => Ok(()),
    }
}

pub(super) fn remove(
    config: &Config,
    file: &dyn SettingsFile,
    named: &str,
    name: &str,
) -> Result<Report, CliError> {
    let filter = config
        .filters
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| {
            CliError::Usage(format!(
                "there is no filter {name}; dam filter list shows the saved filters"
            ))
        })?;
    if filter.runs_a_function {
        return Err(CliError::Usage(format!(
            "filter {name} runs a match function, which is written by hand; \
             remove filter.{name} from config.lua by hand"
        )));
    }
    file.unset(&entry_key("filter", name))?;
    report(&FilterRemoved { name, file: named })
}
