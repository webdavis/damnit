use std::path::Path;

use dam_adapters::{LuaConfigFile, ReadOnly, SqliteStore};
use dam_application::{ConfigFile, Dam, FilterTrial, Progress, Step, UseCaseError, waited_on};

use crate::commands::check_report::{Check, Outcome, Row};
use crate::context::Context;
use crate::error::filter_text::FailureText;
use crate::error::{ConfigReason, UseCaseText};
use crate::output::Format;
use crate::progress::{ProgressOnStandardError, Shown};

const FILTERS: &str = "filters";

pub(super) fn test(
    config_file: &LuaConfigFile,
    store_path: &Path,
    progress: &dyn Progress,
) -> Check {
    let found = |result, detail: &str| Check::named(FILTERS, result, detail);
    let loaded = match config_file.ends_in_lua().and_then(|()| config_file.load()) {
        Ok(loaded) => loaded,
        Err(error) => {
            let detail = format!(
                "not tested, the config did not read: {}",
                ConfigReason(&error)
            );
            return found(Outcome::Issue, &detail).advising("run dam doctor config");
        }
    };
    if loaded.config.filters.is_empty() {
        return none_saved();
    }
    let store = match SqliteStore::open_read_only(store_path) {
        Ok(ReadOnly::Opened(store)) => store,
        Ok(ReadOnly::NoStore) => {
            return found(Outcome::NotRun, "not tested, there is no store yet")
                .advising("run dam status");
        }
        Ok(ReadOnly::LeftOver(_)) | Err(_) => {
            return found(Outcome::Issue, "not tested, the store did not open")
                .advising("run dam doctor database");
        }
    };
    let hidden = ProgressOnStandardError::new(Shown::Hidden, Format::Json);
    let interrupt = Default::default();
    let context = Context::reading(
        Box::new(config_file.clone()),
        loaded,
        Box::new(store),
        hidden,
        &interrupt,
    );
    tried(&context.dam(), progress)
}

pub(super) fn tried(dam: &Dam, progress: &dyn Progress) -> Check {
    if dam.config.filters.is_empty() {
        return none_saved();
    }
    let rows = dam
        .config
        .filters
        .iter()
        .map(|filter| row(dam, &filter.name, progress))
        .collect();
    Check::counting(FILTERS, "filter", rows)
}

fn none_saved() -> Check {
    Check::named(FILTERS, Outcome::Ok, "no saved filters")
}

fn row(dam: &Dam, name: &str, progress: &dyn Progress) -> Row {
    let step = Step {
        name: "try_filter",
        remote: None,
        doing: &format!("trying filter {name}"),
        did: &format!("tried filter {name}"),
        unit: "",
        total: None,
    };
    let tried = waited_on(progress, step, || match dam.try_filter(name) {
        Ok(FilterTrial::Passed { ran_on, kept }) => Ok((ran_on, kept)),
        Ok(FilterTrial::Failed { failure, object }) => Err(match object {
            Some(oid) => format!("failed on {} at {}", oid.short(), FailureText(&failure)),
            None => format!("failed at {}", FailureText(&failure)),
        }),
        Err(error @ UseCaseError::Query(_)) => {
            Err(format!("the query does not parse: {}", UseCaseText(&error)))
        }
        Err(error) => Err(UseCaseText(&error).to_string()),
    });
    match tried {
        Ok((ran_on, kept)) => {
            let objects = if ran_on == 1 { "object" } else { "objects" };
            Row::of(
                name,
                Outcome::Ok,
                format!("ran on {ran_on} {objects}, kept {kept}"),
            )
        }
        Err(detail) => Row::of(name, Outcome::Issue, detail).advising(format!(
            "fix filter.{name} in config.lua, then run dam test filters"
        )),
    }
}
