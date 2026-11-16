use std::rc::Rc;

use clap::Parser;
use dam_application::{RemoteConfig, RemoteName};

use super::run_sync;
use crate::args::{Cli, Command};
use crate::context::Context;
use crate::testing::{RecordingScheduler, context};

struct Shared(Rc<RecordingScheduler>);

impl dam_application::BackgroundSync for Shared {
    fn start(
        &self,
        j: &dam_application::BackgroundJob,
    ) -> Result<dam_application::Change, dam_application::ScheduleError> {
        self.0.start(j)
    }
    fn stop(&self, r: &str) -> Result<dam_application::Change, dam_application::ScheduleError> {
        self.0.stop(r)
    }
    fn enable(
        &self,
        j: &dam_application::BackgroundJob,
    ) -> Result<dam_application::Change, dam_application::ScheduleError> {
        self.0.enable(j)
    }
    fn disable(&self, r: &str) -> Result<dam_application::Change, dam_application::ScheduleError> {
        self.0.disable(r)
    }
}

fn with_todoist(scheduler: &Rc<RecordingScheduler>) -> Context {
    let mut ctx = context();
    ctx.background = Box::new(Shared(scheduler.clone()));
    ctx.config.remotes.push(RemoteConfig {
        name: RemoteName("todoist".into()),
        helper: "t".into(),
        url: "t::".into(),
        credentials: vec![],
        deadline: None,
        sync: Default::default(),
        path: None,
    });
    ctx
}

fn sync(ctx: &Context, line: &str) -> String {
    let words = std::iter::once("dam").chain(line.split(' '));
    let Command::Sync(args) = Cli::try_parse_from(words).unwrap().command else {
        panic!("not sync")
    };
    run_sync(ctx, args).unwrap().human
}

#[test]
fn start_hands_the_scheduler_the_files_in_use_and_the_temporary_interval() {
    let scheduler = Rc::new(RecordingScheduler::default());
    let ctx = with_todoist(&scheduler);
    let shown = sync(&ctx, "sync start todoist --interval 5m");
    assert_eq!(
        shown,
        "todoist: syncing in the background every 5m until the next restart"
    );
    assert_eq!(
        *scheduler.calls.borrow(),
        [
            "start --quiet --config /home/someone/config.lua --store /home/someone/dam.db sync run todoist --when-due --interval 300s"
        ]
    );
}

#[test]
fn disable_now_also_stops_it() {
    let scheduler = Rc::new(RecordingScheduler::default());
    let ctx = with_todoist(&scheduler);
    let shown = sync(&ctx, "sync disable todoist --now");
    assert_eq!(
        shown,
        "todoist: background sync no longer starts at login\ntodoist: background sync stopped"
    );
    assert_eq!(
        *scheduler.calls.borrow(),
        ["disable todoist", "stop todoist"]
    );
}
