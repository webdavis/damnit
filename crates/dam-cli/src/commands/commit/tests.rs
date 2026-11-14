use crate::args::{AddArgs, CommitArgs};
use crate::commands::stage::run_add;
use crate::error::CliError;
use crate::testing::context;
use dam_application::Clock;
use dam_domain::{Object, Oid, Task, Timestamp};
use jiff::civil::Date;

struct TwoPmUtc;
impl Clock for TwoPmUtc {
    fn today(&self) -> Date {
        self.now().to_zoned(jiff::tz::TimeZone::UTC).date()
    }
    fn now(&self) -> Timestamp {
        "2026-10-13T14:00:00Z".parse().unwrap()
    }
    fn zone(&self) -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::UTC
    }
}

#[test]
fn commit_records_the_stage_and_log_lists_it_newest_first() {
    let ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(
            Oid::generate(&mut |x: &mut [u8]| x.fill(1)),
            "a",
        )))
        .unwrap();
    run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    let first = super::run_commit(
        &ctx,
        CommitArgs {
            message: "first".into(),
        },
    )
    .unwrap();
    assert!(first.human.contains("first"));
    assert!(matches!(
        super::run_commit(
            &ctx,
            CommitArgs {
                message: "empty".into()
            }
        ),
        Err(CliError::UseCase(_))
    ));
    let log = super::run_log(&ctx).unwrap();
    assert!(log.human.lines().next().unwrap().contains("first"));
    assert_eq!(log.data["commits"][0]["message"], "first");
    assert_eq!(
        log.data["commits"][0]["changes"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn log_prints_commit_times_in_local_time_like_ls() {
    let mut ctx = context();
    ctx.clock = Box::new(TwoPmUtc);
    ctx.tz = jiff::tz::TimeZone::get("America/New_York").unwrap();
    ctx.store
        .put(&Object::Task(Task::new(
            Oid::generate(&mut |x: &mut [u8]| x.fill(1)),
            "a",
        )))
        .unwrap();
    run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    super::run_commit(
        &ctx,
        CommitArgs {
            message: "standup notes".into(),
        },
    )
    .unwrap();
    let log = super::run_log(&ctx).unwrap();
    let id = log.data["commits"][0]["id"].as_str().unwrap()[..7].to_string();
    assert_eq!(
        log.human,
        format!("{id}  2026-10-13 10:00  1 change  you  standup notes")
    );
    assert_eq!(log.data["commits"][0]["at"], "2026-10-13T14:00:00Z");
}
