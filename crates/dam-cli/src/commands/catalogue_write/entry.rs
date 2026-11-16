use dam_application::{SettingKey, SettingKind};

use crate::error::CliError;

pub(super) fn setting_key(group: &str, name: &str, field: &str) -> Result<SettingKey, CliError> {
    SettingKey::parse(&format!("{group}.{name}.{field}"))
        .map_err(|_| CliError::Usage(format!("a {group} needs a name")))
}

pub(super) fn entry_key(group: &str, name: &str) -> SettingKey {
    SettingKey {
        path: vec![group.to_string(), name.to_string()],
        kind: SettingKind::Text,
    }
}
