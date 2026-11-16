use dam_adapters::Interrupt;

use crate::editor::{EditorSession, EnvEditor};
use crate::output::Format;
use crate::prompt::{Prompt, RefusingPrompt, TerminalPrompt};

pub(crate) fn terminal_pair_for_a_human_and_refusals_for_a_machine(
    format: Format,
    interrupt: &Interrupt,
) -> (Box<dyn Prompt>, Option<Box<dyn EditorSession>>) {
    match format {
        Format::Human | Format::Quiet => (
            Box::new(TerminalPrompt::new(interrupt.clone())),
            Some(Box::new(EnvEditor::from_env())),
        ),
        Format::Json | Format::Toon => (Box::new(RefusingPrompt), None),
    }
}
