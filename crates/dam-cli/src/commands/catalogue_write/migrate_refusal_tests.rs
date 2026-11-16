use dam_application::SetRun;
use dam_application::testing::{FakeSettingsFile, oid};
use dam_domain::{Categories, Category};

use super::migrate_tests::{effort_and_filters, migrating};

#[test]
fn a_map_that_cannot_be_read_is_refused_and_changes_nothing() {
    let ctx = effort_and_filters();
    let file = FakeSettingsFile::default();
    let cases = [
        (
            vec![],
            "name a label to rename: --map OLD=NEW, such as --map quick=light",
        ),
        (
            vec!["quick"],
            "--map takes OLD=NEW, such as --map quick=light",
        ),
        (
            vec!["quick="],
            "--map takes OLD=NEW, such as --map quick=light",
        ),
        (
            vec!["quick=light", "quick=deep"],
            "--map names quick twice; give each old label once",
        ),
        (
            vec!["quick=quick"],
            "--map quick=quick renames quick to itself",
        ),
    ];
    for (map, refused) in cases {
        let error = migrating(&ctx, &file, &map, SetRun::default(), true).unwrap_err();
        assert_eq!(error.to_string(), refused);
    }
    assert!(file.written.borrow().is_empty());
    assert!(
        ctx.store
            .get(&oid(1))
            .unwrap()
            .unwrap()
            .base()
            .labels
            .contains("quick")
    );
}

#[test]
fn a_new_label_of_another_category_is_refused() {
    let mut ctx = effort_and_filters();
    ctx.config.categories = Categories::new(vec![
        Category {
            name: "effort".into(),
            values: vec!["quick".into()],
            exclusive: true,
        },
        Category {
            name: "place".into(),
            values: vec!["home".into()],
            exclusive: false,
        },
    ])
    .unwrap();
    let error = migrating(
        &ctx,
        &FakeSettingsFile::default(),
        &["quick=home"],
        SetRun::default(),
        false,
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "quick belongs to category effort and home to category place; a label belongs to one category, \
         so rename quick to a label of effort or to a free one (dam category list shows each one's labels)"
    );
}

#[test]
fn a_request_names_the_map_as_an_object() {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        file.path(),
        r#"{"map": {"quick": "light"}, "preview": true, "expect": 2, "update_config": true}"#,
    )
    .unwrap();
    let line = ["dam", "category", "migrate", "--request"]
        .map(std::ffi::OsString::from)
        .into_iter()
        .chain([file.path().as_os_str().to_owned()])
        .collect();
    let cli = crate::request::parse(line).unwrap_or_else(|refused| panic!("{}", refused.error));
    let crate::args::Command::Category(args) = cli.command else {
        panic!("not category")
    };
    let crate::args::CategoryCommand::Migrate {
        map,
        preview,
        expect,
        update_config,
    } = args.command
    else {
        panic!("not migrate")
    };
    assert_eq!(
        (map, preview, expect, update_config),
        (vec!["quick=light".to_string()], true, Some(2), true)
    );
}

#[test]
fn the_answer_matches_its_schema() {
    let ctx = effort_and_filters();
    let report = migrating(
        &ctx,
        &FakeSettingsFile::default(),
        &["quick=light"],
        SetRun::default(),
        false,
    )
    .unwrap();
    let mut options = jsonschema::options();
    for text in [
        include_str!("../../../../../docs/schemas/changes.schema.json"),
        include_str!("../../../../../docs/schemas/status.schema.json"),
        include_str!("../../../../../docs/schemas/object.schema.json"),
    ] {
        let schema: serde_json::Value = serde_json::from_str(text).unwrap();
        let id = schema["$id"].as_str().unwrap().to_string();
        options = options.with_resource(id, jsonschema::Resource::from_contents(schema).unwrap());
    }
    let schema = serde_json::from_str(include_str!(
        "../../../../../docs/schemas/category-migrate.schema.json"
    ))
    .unwrap();
    let validator = options.build(&schema).unwrap();
    assert!(validator.is_valid(&report.data), "{}", report.data);
}
