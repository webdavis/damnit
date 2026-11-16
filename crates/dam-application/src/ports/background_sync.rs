use std::time::Duration;

pub trait BackgroundSync {
    fn start(&self, job: &BackgroundJob) -> Result<Change, ScheduleError>;
    fn stop(&self, remote: &str) -> Result<Change, ScheduleError>;
    fn enable(&self, job: &BackgroundJob) -> Result<Change, ScheduleError>;
    fn disable(&self, remote: &str) -> Result<Change, ScheduleError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundJob {
    pub remote: String,
    pub dam: String,
    pub config: String,
    pub store: String,
    pub search_path: String,
    pub interval: Option<Duration>,
}

impl BackgroundJob {
    pub fn label(&self) -> String {
        label_of(&self.remote)
    }

    pub fn arguments(&self) -> Vec<String> {
        let mut words: Vec<String> = [
            "--quiet",
            "--config",
            &self.config,
            "--store",
            &self.store,
            "sync",
            "run",
            &self.remote,
            "--when-due",
        ]
        .map(String::from)
        .into();
        if let Some(interval) = self.interval {
            words.push("--interval".into());
            words.push(format!("{}s", interval.as_secs()));
        }
        words
    }
}

pub fn label_of(remote: &str) -> String {
    format!("dam-sync.{remote}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Now,
    Already,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ScheduleError {
    NoScheduler { crontab_line: String },
    Failed(String),
}
