use std::collections::BTreeSet;

use dam_application::{Config, SettingValue, SettingsFile, Store};
use dam_domain::{Categories, Category, CategoryError, Object};

use super::category_written::{CategoryCreated, CategoryRemoved};
use super::entry::{entry_key, setting_key};
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;

pub(super) fn create(
    config: &Config,
    file: &dyn SettingsFile,
    named: &str,
    yours: &Categories,
    (name, values, exclusive): (&str, &str, bool),
) -> Result<Report, CliError> {
    let values_key = setting_key("category", name, "values")?;
    let exists = || {
        CliError::Usage(format!(
            "category {name} exists; change its values with dam config set category.{name}.values '<values>'"
        ))
    };
    if config.categories.category_of_name(name).is_some() {
        return Err(exists());
    }
    let values: Vec<String> = values
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect();
    let made = Category {
        name: name.to_string(),
        values: values.clone(),
        exclusive,
    };
    let mut categories = config.categories.all().to_vec();
    categories.push(made.clone());
    Categories::new(categories).map_err(|error| match error {
        CategoryError::EmptyCategory(_) => CliError::Usage(format!(
            "category {name} needs at least one label in --values"
        )),
        CategoryError::DuplicateName(_) => exists(),
        CategoryError::DuplicateValue { value, first, .. } if first == name => {
            CliError::Usage(format!("--values names {value} twice"))
        }
        CategoryError::DuplicateValue { value, first, .. } => CliError::Usage(format!(
            "{value} already belongs to category {first}; a label belongs to one category, \
             and dam category list shows each one's labels"
        )),
    })?;
    file.set(&values_key, &SettingValue::List(values.clone()))?;
    if exclusive {
        file.set(
            &setting_key("category", name, "exclusive")?,
            &SettingValue::Flag(true),
        )?;
    }
    let made = Categories::new(vec![made]).unwrap_or_default();
    let sets_aside = yours
        .set_aside_by(&made)
        .into_iter()
        .map(|(yours, _)| yours);
    report(&CategoryCreated {
        name,
        sets_aside: sets_aside.collect(),
        values: &values,
        exclusive,
        file: named,
    })
}

pub(super) fn remove(
    config: &Config,
    store: &dyn Store,
    file: &dyn SettingsFile,
    named: &str,
    name: &str,
) -> Result<Report, CliError> {
    let category = config.categories.category_of_name(name).ok_or_else(|| {
        CliError::Usage(format!(
            "there is no category {name}; dam category list shows the categories"
        ))
    })?;
    let labels: BTreeSet<&String> = category.values.iter().collect();
    let tasks_keeping_labels = store
        .all()?
        .iter()
        .filter(|object| match object {
            Object::Task(task) => task.base.labels.iter().any(|l| labels.contains(l)),
            _ => false,
        })
        .count();
    file.unset(&entry_key("category", name))?;
    report(&CategoryRemoved {
        name,
        tasks_keeping_labels,
        file: named,
    })
}
