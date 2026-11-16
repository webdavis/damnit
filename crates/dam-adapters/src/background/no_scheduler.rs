use dam_application::{BackgroundJob, BackgroundState, BackgroundSync, Change, ScheduleError};

pub(super) struct NoScheduler;

impl NoScheduler {
    fn refused(job: &BackgroundJob) -> Result<Change, ScheduleError> {
        let words = std::iter::once(job.dam.clone()).chain(job.arguments());
        let command: Vec<String> = words.map(|w| quoted(&w)).collect();
        Err(ScheduleError::NoScheduler {
            crontab_line: format!("* * * * * {}", command.join(" ")),
        })
    }

    fn by_hand(remote: &str) -> Result<Change, ScheduleError> {
        Err(ScheduleError::Failed(format!(
            "this machine has neither launchd nor systemd; remove the line that runs dam sync run {remote} with crontab -e"
        )))
    }
}

fn quoted(word: &str) -> String {
    let plain = |c: char| c.is_ascii_alphanumeric() || "/._-:=".contains(c);
    match word.chars().all(plain) {
        true => word.to_string(),
        false => format!("'{}'", word.replace('\'', r"'\''")),
    }
}

impl BackgroundSync for NoScheduler {
    fn start(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        NoScheduler::refused(job)
    }
    fn stop(&self, remote: &str) -> Result<Change, ScheduleError> {
        NoScheduler::by_hand(remote)
    }
    fn enable(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        NoScheduler::refused(job)
    }
    fn disable(&self, remote: &str) -> Result<Change, ScheduleError> {
        NoScheduler::by_hand(remote)
    }
    fn state(&self, _: &str) -> Result<BackgroundState, ScheduleError> {
        Ok(BackgroundState::default())
    }
}

#[cfg(test)]
mod tests;
