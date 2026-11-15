use dam_application::{Installed, Removed, Schedule, ScheduleError, ScheduledRun};

pub struct NoScheduler;

impl Schedule for NoScheduler {
    fn installed(&self) -> bool {
        false
    }

    fn install(&self, run: &ScheduledRun) -> Result<Installed, ScheduleError> {
        let command: Vec<String> = std::iter::once(run.dam.as_str())
            .chain(run.arguments())
            .map(quoted)
            .collect();
        let crontab_line = format!(
            "* * * * * PATH={} XDG_STATE_HOME={} {} >/dev/null 2>&1",
            quoted(&run.search_path),
            quoted(&run.state_home),
            command.join(" ")
        );
        Err(ScheduleError::NoScheduler { crontab_line })
    }

    fn remove(&self) -> Result<Removed, ScheduleError> {
        Ok(Removed::NotInstalled)
    }
}

fn quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''").replace('%', "\\%"))
}

#[cfg(test)]
mod tests;
