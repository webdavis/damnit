use dam_adapters::LuaConfigFile;
use dam_application::{ConfigFile, Refusal};
use serde::Serialize;

use super::setting::Setting;
use super::settings::settings;
use super::warnings::warnings_naming_only_the_token;
use crate::error::CliError;
use crate::output::Report;
use crate::style::{column, field};
use crate::view::{Human, View, report};

#[derive(Serialize)]
#[serde(transparent)]
struct Got<'a> {
    #[serde(skip)]
    key: &'a str,
    setting: &'a Setting,
    #[serde(skip)]
    warnings: Vec<String>,
}

impl View for Got<'_> {
    fn human(&self) -> Human {
        let rows = self.setting.rows(self.key);
        let column = column(rows.iter().map(|(key, _)| key.as_str()));
        Human::styled(|paint| {
            rows.iter()
                .map(|(key, value)| field(paint, column, key, value))
                .collect()
        })
    }

    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

pub(super) fn run(file: &LuaConfigFile, key: &str) -> Result<Report, CliError> {
    let config = file.load()?.config;
    let settings = settings(&config);
    match settings.readings(key).as_slice() {
        [] => Err(Refusal::NoSuchSetting(key.to_string()).into()),
        [(_, setting)] => report(&Got {
            key,
            setting,
            warnings: warnings_naming_only_the_token(&config.tokens),
        }),
        readings => Err(CliError::Usage(read_more_than_one_way(key, readings))),
    }
}

fn read_more_than_one_way(key: &str, readings: &[(Vec<&str>, &Setting)]) -> String {
    let mut paths: Vec<String> = readings.iter().map(|(path, _)| path.join(" / ")).collect();
    let last = paths.pop().unwrap_or_default();
    let (count, every) = match readings.len() {
        2 => ("two".to_string(), "both"),
        n => (n.to_string(), "all of them"),
    };
    let first = readings
        .first()
        .map_or(&[][..], |(path, _)| path.as_slice());
    let parent: Vec<&str> = first
        .iter()
        .enumerate()
        .take_while(|(at, name)| {
            readings
                .iter()
                .all(|(path, _)| path.get(*at) == Some(*name))
        })
        .map(|(_, name)| *name)
        .collect();
    format!(
        "{key} names {count} settings, {} and {last}; run dam config get {} to see {every}",
        paths.join(", "),
        parent.join(".")
    )
}
