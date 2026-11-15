use std::path::PathBuf;

use dam_application::{Config, Installed, Removed};
use serde::Serialize;

use crate::style::{self, Mark};
use crate::view::{Human, View};

const RUNS: &str = "dam sync run starts every minute";

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Enabling {
    Installed,
    AlreadyInstalled,
}

#[derive(Serialize)]
pub(super) struct Enabled<'a> {
    schedule: Enabling,
    files: Vec<String>,
    last_run: &'a str,
    remotes: Vec<Syncing<'a>>,
}

#[derive(Serialize)]
struct Syncing<'a> {
    remote: &'a str,
    sync_every: &'a str,
}

impl<'a> Enabled<'a> {
    pub(super) fn new(installed: Installed, last_run: &'a str, config: &'a Config) -> Enabled<'a> {
        let (schedule, files) = match installed {
            Installed::Now(files) => (Enabling::Installed, files),
            Installed::Already(files) => (Enabling::AlreadyInstalled, files),
        };
        let remotes = config.remotes.iter().filter_map(|r| {
            let every = r.sync_every.as_ref()?;
            Some(Syncing {
                remote: &r.name.0,
                sync_every: &every.text,
            })
        });
        Enabled {
            schedule,
            files: shown(files),
            last_run,
            remotes: remotes.collect(),
        }
    }
}

impl View for Enabled<'_> {
    fn human(&self) -> Human {
        Human::styled(|paint| {
            let (mark, said, verb) = match self.schedule {
                Enabling::Installed => (Mark::Good, "installed", "wrote"),
                Enabling::AlreadyInstalled => (Mark::Quiet, "already installed", "kept"),
            };
            let mut lines = vec![style::row(
                paint,
                mark,
                0,
                &format!("schedule {said}: {RUNS}"),
            )];
            lines.extend(self.files.iter().map(|f| format!("  {verb} {f}")));
            lines.push(format!("  last run {}", self.last_run));
            lines.extend(self.remotes.iter().map(|r| {
                let text = format!("{} syncs every {}", r.remote, r.sync_every);
                style::row(paint, Mark::Quiet, 0, &text)
            }));
            if self.remotes.is_empty() {
                let text = "no remote has sync_every; set it on a remote in config.lua";
                lines.push(style::row(paint, Mark::Warn, 0, text));
            }
            lines
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Disabling {
    Removed,
    NotInstalled,
}

#[derive(Serialize)]
pub(super) struct Disabled {
    schedule: Disabling,
    files: Vec<String>,
}

impl Disabled {
    pub(super) fn new(removed: Removed) -> Disabled {
        let (schedule, files) = match removed {
            Removed::Now(files) => (Disabling::Removed, shown(files)),
            Removed::NotInstalled => (Disabling::NotInstalled, Vec::new()),
        };
        Disabled { schedule, files }
    }
}

impl View for Disabled {
    fn human(&self) -> Human {
        Human::styled(|paint| match self.schedule {
            Disabling::Removed => {
                std::iter::once(style::row(paint, Mark::Good, 0, "schedule removed"))
                    .chain(self.files.iter().map(|f| format!("  removed {f}")))
                    .collect()
            }
            Disabling::NotInstalled => {
                vec![style::row(paint, Mark::Quiet, 0, "schedule not installed")]
            }
        })
    }
}

fn shown(files: Vec<PathBuf>) -> Vec<String> {
    files.iter().map(|f| f.display().to_string()).collect()
}
