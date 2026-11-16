use dam_application::{SettingKey, SettingKind, SettingValue, SettingsFile};
use dam_domain::{FilterTable, TableValue};

use super::support::config;

fn entry(name: &str) -> SettingKey {
    SettingKey {
        path: vec!["filter".into(), name.into()],
        kind: SettingKind::Text,
    }
}

fn table(fields: &[(&str, TableValue)], any: Vec<FilterTable>) -> FilterTable {
    FilterTable {
        fields: fields
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
        any,
    }
}

#[test]
fn a_filter_written_as_a_table_reads_back_the_same() {
    let folder = config("-- mine\nreturn {\n  owner = \"Ada\",\n}\n");
    let written = table(
        &[
            (
                "labels",
                TableValue::Many(vec!["bug".into(), "not:\"waiting\"".into()]),
            ),
            ("focus.day", TableValue::One("today".into())),
        ],
        vec![
            table(&[("due", TableValue::One("..today".into()))], vec![]),
            table(&[("priority", TableValue::One("1,2".into()))], vec![]),
        ],
    );
    folder
        .file()
        .set(&entry("soon"), &SettingValue::Table(written.clone()))
        .unwrap();
    assert!(folder.text("config.lua").starts_with("-- mine\n"));
    assert_eq!(
        folder.load().unwrap().filter("soon").unwrap().table,
        written
    );
}
