use dam_adapters::LuaConfigFile;

use crate::output::{Format, shown};

fn config_folder() -> (tempfile::TempDir, LuaConfigFile) {
    let dir = tempfile::tempdir().unwrap();
    let file = LuaConfigFile::new(dir.path().join("config.lua"));
    (dir, file)
}

#[test]
fn a_set_setting_is_what_get_reads() {
    let (_dir, file) = config_folder();
    super::set::run(&file, "progress", Some("false")).unwrap();
    let got = super::get::run(&file, "progress").unwrap();
    assert_eq!(
        got.data,
        serde_json::json!({"value": false, "default": false})
    );
}

#[test]
fn quiet_prints_nothing_for_a_set() {
    let (_dir, file) = config_folder();
    let report = super::set::run(&file, "owner", Some("Grace")).unwrap();
    assert_eq!(shown(Format::Quiet, false, None, &report).unwrap(), None);
    assert!(report.standard_error_lines.is_empty());
}
