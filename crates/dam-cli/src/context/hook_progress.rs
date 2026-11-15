use dam_application::{HookEvent, HookProgress};

use crate::output::Format;

pub(crate) struct HookProgressOnStandardError {
    in_plain_text: bool,
}

impl HookProgressOnStandardError {
    pub(crate) fn for_format(format: Format) -> HookProgressOnStandardError {
        HookProgressOnStandardError {
            in_plain_text: format == Format::Human,
        }
    }

    fn line(&self, event: HookEvent, tasks: usize) -> Option<String> {
        self.in_plain_text
            .then(|| format!("dam: running {} for {tasks} tasks", event.as_str()))
    }
}

impl HookProgress for HookProgressOnStandardError {
    fn running(&self, event: HookEvent, tasks: usize) {
        if let Some(line) = self.line(event, tasks) {
            eprintln!("{line}");
        }
    }
}

#[cfg(test)]
mod tests;
