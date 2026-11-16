use dam_application::testing::{FakeSettingsFile, Written, oid};
use dam_application::{FilterConfig, FilterRule, SetRun, SettingValue};
use dam_domain::{Categories, Category, Object, Task};

use super::migrate::{Migration, migrate};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::testing::context;

pub(super) fn effort_and_filters() -> Context {
    let mut ctx = context();
    ctx.config.categories = Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["quick".into(), "deep".into()],
        exclusive: true,
    }])
    .unwrap();
    ctx.config.filters = vec![
        FilterConfig {
            name: "short".into(),
            rule: FilterRule::Query("@quick & !done".into()),
        },
        FilterConfig {
            name: "picked".into(),
            rule: FilterRule::QueryThenFunction("effort:quick".into()),
        },
    ];
    for byte in [1, 2] {
        let mut task = Task::new(oid(byte), format!("t{byte}"));
        task.base.labels = ["quick".to_string()].into();
        ctx.store.put(&Object::Task(task)).unwrap();
    }
    ctx
}

pub(super) fn migrating(
    ctx: &Context,
    file: &FakeSettingsFile,
    map: &[&str],
    run: SetRun,
    update: bool,
) -> Result<Report, CliError> {
    let map: Vec<String> = map.iter().map(|m| m.to_string()).collect();
    migrate(
        &ctx.dam(),
        &ctx.config,
        file,
        Migration {
            map: &map,
            run,
            update_config: update,
        },
    )
}

fn tail(report: &Report, lines: usize) -> Vec<&str> {
    let all: Vec<&str> = report.human.lines().collect();
    all[all.len() - lines..].to_vec()
}

#[test]
fn by_default_the_tasks_change_and_each_config_place_is_listed_with_its_fix() {
    let ctx = effort_and_filters();
    let file = FakeSettingsFile::default();
    let report = migrating(&ctx, &file, &["quick=light"], SetRun::default(), false).unwrap();
    assert!(file.written.borrow().is_empty());
    assert_eq!(
        tail(&report, 5),
        vec![
            "changed 2 of 2",
            "config.lua still names an old label in 3 places:",
            "  category effort → dam config set category.effort.values 'light, deep'",
            "  filter short → dam config set filter.short.query '@light & !done'",
            "  filter picked runs a match function → edit filter.picked in config.lua by hand",
        ]
    );
    assert_eq!(
        report.data["config"][0],
        serde_json::json!({"key": "category.effort.values", "value": ["light", "deep"], "written": false})
    );
    assert_eq!(
        report.data["config"][2],
        serde_json::json!({"key": "filter.picked", "by_hand": true, "written": false})
    );
}

#[test]
fn update_config_rewrites_each_place_it_can_after_the_tasks_change() {
    let ctx = effort_and_filters();
    let file = FakeSettingsFile::default();
    let report = migrating(&ctx, &file, &["quick=light"], SetRun::default(), true).unwrap();
    assert_eq!(
        *file.written.borrow(),
        vec![
            Written::Set(
                "category.effort.values".into(),
                SettingValue::List(vec!["light".into(), "deep".into()])
            ),
            Written::Set(
                "filter.short.query".into(),
                SettingValue::Text("@light & !done".into())
            ),
        ]
    );
    assert_eq!(
        tail(&report, 4),
        vec![
            "config.lua updated in 2 places:",
            "  category.effort.values = light, deep",
            "  filter.short.query = @light & !done",
            "  filter picked runs a match function → edit filter.picked in config.lua by hand",
        ]
    );
    assert_eq!(report.data["config"][1]["written"], true);
}

#[test]
fn a_preview_with_update_config_shows_what_the_config_would_become_and_writes_nothing() {
    let ctx = effort_and_filters();
    let file = FakeSettingsFile::default();
    let run = SetRun {
        preview: true,
        expect: None,
    };
    let report = migrating(&ctx, &file, &["quick=light"], run, true).unwrap();
    assert!(file.written.borrow().is_empty());
    assert!(report.human.contains(
        "preview: 2 of 2 would change, nothing written\nconfig.lua would change in 2 places:"
    ));
    assert_eq!(report.data["preview"], true);
}

#[test]
fn a_config_write_that_fails_says_the_tasks_changed_and_what_to_set_by_hand() {
    let ctx = effort_and_filters();
    let file = FakeSettingsFile::refusing();
    let error = migrating(&ctx, &file, &["quick=light"], SetRun::default(), true).unwrap_err();
    assert_eq!(
        error.to_string(),
        "the labels changed on 2 objects, but config.lua was not written: \
         dam config set category.effort.values 'light, deep' \
         and dam config set filter.short.query '@light & !done' finish it"
    );
    let task = ctx.store.get(&oid(1)).unwrap().unwrap();
    assert!(task.base().labels.contains("light"));
}
