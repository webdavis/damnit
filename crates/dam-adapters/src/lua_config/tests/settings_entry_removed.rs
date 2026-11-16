use dam_application::{SettingKey, SettingKind, SettingsFile};

use super::support::config;

fn entry(group: &str, name: &str) -> SettingKey {
    SettingKey {
        path: vec![group.into(), name.into()],
        kind: SettingKind::Text,
    }
}

#[test]
fn unsetting_an_entry_removes_the_whole_table_and_leaves_the_rest_byte_for_byte() {
    let folder = config(
        "-- mine\nreturn {\n  filter = {\n    soon = { due = \"today\" }, -- near\n    \
         later = { due = \"today+1\" },\n  },\n  category = {\n    \
         effort = { values = { \"light\", \"deep\" }, exclusive = true },\n  },\n}\n",
    );
    let change = folder.file().unset(&entry("filter", "soon")).unwrap();
    assert_eq!(change.key, "filter.soon");
    assert_eq!(change.old.as_deref(), Some("today"));
    folder.file().unset(&entry("category", "effort")).unwrap();
    assert_eq!(
        folder.text("config.lua"),
        "-- mine\nreturn {\n  filter = {\n    later = { due = \"today+1\" },\n  },\n  \
         category = {\n  },\n}\n"
    );
    let loaded = folder.load().unwrap();
    assert_eq!(loaded.filters.len(), 1);
    assert!(loaded.categories.all().is_empty());
}
