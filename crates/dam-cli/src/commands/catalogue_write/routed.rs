use dam_application::{Config, SettingsFile, Store};
use dam_domain::Categories;

use super::{category as categories, filter as filters};
use crate::args::{CategoryCommand, FilterCommand};
use crate::commands::config::route::{Written, written};
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;

pub(crate) struct Settings<'a> {
    pub(crate) config: &'a Config,
    pub(crate) personal: &'a dyn SettingsFile,
    pub(crate) standing: &'a Standing,
}

impl Settings<'_> {
    fn routed(
        &self,
        key: &str,
        global: bool,
        verb: &str,
        run: impl FnOnce(&Config, &dyn SettingsFile, &str, &Categories) -> Result<Report, CliError>,
    ) -> Result<Report, CliError> {
        let target = written(key, global, self.standing, verb)?;
        let named = target.name();
        match (&target, self.standing) {
            (Written::Project(file), Standing::Inside(project)) => {
                let rules = &project.settings;
                let config = Config {
                    filters: rules.filters.clone(),
                    categories: rules.categories.clone(),
                    ..Config::default()
                };
                run(&config, file, named, &self.config.categories)
            }
            _ => run(self.config, self.personal, named, &Categories::default()),
        }
    }
}

pub(crate) fn filter(settings: &Settings, command: FilterCommand) -> Result<Report, CliError> {
    match command {
        FilterCommand::Create {
            name,
            query,
            global,
        } => settings.routed(
            &format!("filter.{name}.query"),
            global,
            "filter create",
            |config, file, named, _| filters::create(config, file, named, &name, &query),
        ),
        FilterCommand::Remove { name, global } => settings.routed(
            &format!("filter.{name}.query"),
            global,
            "filter remove",
            |config, file, named, _| filters::remove(config, file, named, &name),
        ),
        FilterCommand::List => unreachable!("dam filter list reads the config alone"),
    }
}

pub(crate) fn category(
    settings: &Settings,
    store: &dyn Store,
    command: CategoryCommand,
) -> Result<Report, CliError> {
    match command {
        CategoryCommand::Create {
            name,
            values,
            exclusive,
            global,
        } => settings.routed(
            &format!("category.{name}.values"),
            global,
            "category create",
            |config, file, named, yours| {
                categories::create(config, file, named, yours, (&name, &values, exclusive))
            },
        ),
        CategoryCommand::Remove { name, global } => settings.routed(
            &format!("category.{name}.values"),
            global,
            "category remove",
            |config, file, named, _| categories::remove(config, store, file, named, &name),
        ),
        _ => unreachable!("dam category migrate and list are answered elsewhere"),
    }
}
