use std::path::PathBuf;

pub trait Schedule {
    fn installed(&self) -> bool;
    fn install(&self, run: &ScheduledRun) -> Result<Installed, ScheduleError>;
    fn remove(&self) -> Result<Removed, ScheduleError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledRun {
    pub dam: String,
    pub config: String,
    pub store: String,
    pub state_home: String,
    pub search_path: String,
}

impl ScheduledRun {
    pub fn arguments(&self) -> [&str; 7] {
        [
            "--config",
            &self.config,
            "--store",
            &self.store,
            "sync",
            "run",
            "--record-last-run",
        ]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Installed {
    Now(Vec<PathBuf>),
    Already(Vec<PathBuf>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Removed {
    Now(Vec<PathBuf>),
    NotInstalled,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ScheduleError {
    NoScheduler { crontab_line: String },
    Failed(String),
}
