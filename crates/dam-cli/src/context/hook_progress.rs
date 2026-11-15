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
}

impl HookProgress for HookProgressOnStandardError {
    fn running(&self, event: HookEvent, tasks: usize) {
        if self.in_plain_text {
            eprintln!("dam: running {} for {tasks} tasks", event.as_str());
        }
    }
}
