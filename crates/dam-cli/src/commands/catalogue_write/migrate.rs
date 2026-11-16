use dam_application::{
    Config, ConfigPlace, Dam, LabelMap, MapProblem, SetRun, SettingValue, SettingsFile,
    config_places,
};

use super::entry::{entry_key, setting_key};
use super::migrated::{Migrated, Place, fix};
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;

pub(super) struct Migration<'a> {
    pub(super) map: &'a [String],
    pub(super) run: SetRun,
    pub(super) update_config: bool,
}

pub(super) fn migrate(
    dam: &Dam,
    config: &Config,
    file: &dyn SettingsFile,
    migration: Migration,
) -> Result<Report, CliError> {
    let map = label_map(config, migration.map)?;
    let changes = dam.migrate_labels(&map, migration.run)?;
    let writing = migration.update_config && !migration.run.preview;
    let mut places: Vec<Place> = config_places(config, &map)
        .into_iter()
        .map(Place::from)
        .collect();
    if writing {
        write(file, &mut places, changes.oids.len())?;
    }
    report(&Migrated::of(
        &changes,
        config.owner_name(),
        dam,
        places,
        migration.update_config,
    ))
}

fn write(file: &dyn SettingsFile, places: &mut [Place], changed: usize) -> Result<(), CliError> {
    for at in 0..places.len() {
        let Some(value) = setting_value(&places[at].place) else {
            continue;
        };
        let key = setting_key_of(&places[at].place)?;
        if file.set(&key, &value).is_err() {
            let left: Vec<String> = places[at..].iter().filter_map(|p| fix(&p.place)).collect();
            return Err(CliError::Io(format!(
                "the labels changed on {changed} objects, but config.lua was not written: {} finish it",
                left.join(" and ")
            )));
        }
        places[at].written = true;
    }
    Ok(())
}

fn setting_value(place: &ConfigPlace) -> Option<SettingValue> {
    match place {
        ConfigPlace::Category { values, .. } => Some(SettingValue::List(values.clone())),
        ConfigPlace::Filter { table, .. } => Some(SettingValue::Table(table.clone())),
        ConfigPlace::HandWritten { .. } => None,
    }
}

fn setting_key_of(place: &ConfigPlace) -> Result<dam_application::SettingKey, CliError> {
    match place {
        ConfigPlace::Category { name, .. } => setting_key("category", name, "values"),
        ConfigPlace::Filter { name, .. } | ConfigPlace::HandWritten { name } => {
            Ok(entry_key("filter", name))
        }
    }
}

fn label_map(config: &Config, map: &[String]) -> Result<LabelMap, CliError> {
    let shape = || CliError::Usage("--map takes OLD=NEW, such as --map quick=light".into());
    let pairs = map
        .iter()
        .map(|text| match text.split_once('=') {
            Some((old, new)) if !old.is_empty() && !new.is_empty() => {
                Ok((old.to_string(), new.to_string()))
            }
            _ => Err(shape()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    LabelMap::new(pairs, &config.categories).map_err(|problem| match problem {
        MapProblem::Empty => CliError::Usage(
            "name a label to rename: --map OLD=NEW, such as --map quick=light".into(),
        ),
        MapProblem::Blank => shape(),
        MapProblem::Twice(old) => {
            CliError::Usage(format!("--map names {old} twice; give each old label once"))
        }
        MapProblem::Same(old) => {
            CliError::Usage(format!("--map {old}={old} renames {old} to itself"))
        }
        MapProblem::TwoCategories {
            old,
            new,
            old_category,
            new_category,
        } => CliError::Usage(format!(
            "{old} belongs to category {old_category} and {new} to category {new_category}; \
             a label belongs to one category, so rename {old} to a label of {old_category} \
             or to a free one (dam category list shows each one's labels)"
        )),
    })
}
