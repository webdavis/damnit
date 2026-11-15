use crate::args::StatusArgs;
use crate::context::Context;
use crate::testing::{context, holding};
use dam_application::testing::contract::scheduled_run;
use dam_application::{ConfiguredDuration, RemoteConfig, RemoteName};
use dam_domain::Permission::{ConfigRead, RemoteRead};

const NOT_INSTALLED: &str =
    "warning: sync_every is set on todoist and the schedule is not installed; run dam sync enable";

fn syncing_every(every: Option<&str>) -> Context {
    let mut ctx = context();
    ctx.config.remotes = vec![RemoteConfig {
        name: RemoteName("todoist".into()),
        helper: "todoist".into(),
        url: "todoist::".into(),
        credentials: vec![],
        deadline: None,
        sync_every: every.map(|text| ConfiguredDuration::parse(text).unwrap()),
        path: None,
    }];
    ctx
}

fn warnings(ctx: &Context) -> Vec<String> {
    let report = super::run_status(ctx, StatusArgs { full: false }).unwrap();
    let warnings = report.data["warnings"].as_array().unwrap().iter();
    let warnings: Vec<String> = warnings.map(|w| w.as_str().unwrap().into()).collect();
    for warning in &warnings {
        assert!(report.human.contains(warning.as_str()), "{}", report.human);
    }
    warnings
}

#[test]
fn sync_every_without_the_schedule_warns_and_names_dam_sync_enable() {
    assert_eq!(warnings(&syncing_every(Some("15m"))), [NOT_INSTALLED]);
}

#[test]
fn no_warning_once_the_schedule_is_installed_or_without_sync_every() {
    let ctx = syncing_every(Some("15m"));
    ctx.schedule.install(&scheduled_run()).unwrap();
    assert_eq!(warnings(&ctx), Vec::<String>::new());
    assert_eq!(warnings(&syncing_every(None)), Vec::<String>::new());
}

#[test]
fn a_token_is_not_told_of_the_schedule() {
    let mut ctx = syncing_every(Some("15m"));
    ctx.access = holding(&[ConfigRead, RemoteRead]);
    assert_eq!(warnings(&ctx), Vec::<String>::new());
}
