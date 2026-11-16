use dam_adapters::LuaConfigFile;
use dam_application::{KeyRefusal, SettingChange, SettingKey, SettingKind, SettingsFile};
use serde::Serialize;

use crate::error::CliError;
use crate::output::Report;
use crate::style::{column, field};
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct Changed<'a>(#[serde(with = "ChangeDoc")] &'a SettingChange);

#[derive(Serialize)]
#[serde(remote = "SettingChange")]
struct ChangeDoc {
    key: String,
    old: Option<String>,
    new: Option<String>,
}

impl View for Changed<'_> {
    fn human(&self) -> Human {
        let shown = |value: &Option<String>| value.clone().unwrap_or_else(|| "not set".into());
        let rows = [
            ("was", shown(&self.0.old)),
            (self.0.key.as_str(), shown(&self.0.new)),
        ];
        let column = column(rows.iter().map(|(key, _)| *key));
        Human::styled(|paint| {
            rows.iter()
                .map(|(key, value)| field(paint, column, key, value))
                .collect()
        })
    }
}

pub(super) fn run(
    file: &LuaConfigFile,
    key: &str,
    value: Option<&str>,
) -> Result<Report, CliError> {
    let key = SettingKey::parse(key).map_err(refused)?;
    let change = match value {
        Some(text) => {
            let value = key.value(text).map_err(|bad| {
                CliError::Usage(format!(
                    "{} takes {}, and {:?} is not one",
                    bad.key,
                    kind_text(bad.expected),
                    bad.value
                ))
            })?;
            file.set(&key, &value)?
        }
        None => file.unset(&key)?,
    };
    report(&Changed(&change))
}

fn refused(refusal: KeyRefusal) -> CliError {
    CliError::Usage(match refusal {
        KeyRefusal::Unknown(key) => format!(
            "dam has no setting {key}; dam config list shows the keys, and docs/lua.md lists every one"
        ),
        KeyRefusal::Secret(key) => {
            format!("{key} belongs in secrets.lua, and dam config set never writes a secret")
        }
        KeyRefusal::Function(key) => {
            format!("{key} is a function; write it in config.lua by hand")
        }
    })
}

fn kind_text(kind: SettingKind) -> &'static str {
    match kind {
        SettingKind::Text => "text",
        SettingKind::Duration => "a duration, a whole number followed by s, m or h",
        SettingKind::Flag => "true or false",
        SettingKind::List => "a list of items separated by commas",
    }
}
