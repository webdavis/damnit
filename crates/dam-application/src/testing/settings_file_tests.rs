use super::{FakeSettingsFile, Written};
use crate::ports::{ConfigError, SettingsFile};
use crate::setting::{SettingKey, SettingValue};

fn owner() -> SettingKey {
    SettingKey::parse("owner").unwrap()
}

#[test]
fn the_fake_records_each_set_and_unset_and_reports_the_old_value() {
    let file = FakeSettingsFile::default();
    let ada = SettingValue::Text("Ada".into());
    file.set(&owner(), &ada).unwrap();
    let change = file.unset(&owner()).unwrap();
    assert_eq!(change.old.as_deref(), Some("Ada"));
    assert_eq!(change.new, None);
    assert_eq!(
        file.written.into_inner(),
        [
            Written::Set("owner".into(), ada),
            Written::Unset("owner".into())
        ]
    );
}

#[test]
fn a_refusing_fake_writes_nothing() {
    let file = FakeSettingsFile::refusing();
    let error = file.unset(&owner()).unwrap_err();
    assert!(matches!(error, ConfigError::SettingReadOnly { .. }));
    assert!(file.written.borrow().is_empty());
}
