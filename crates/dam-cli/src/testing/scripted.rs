use std::cell::RefCell;

use crate::editor::{EditorError, EditorSession};
use crate::error::CliError;
use crate::prompt::Prompt;

pub(crate) struct ScriptedPrompt {
    pub(crate) choices: RefCell<Vec<usize>>,
    pub(crate) texts: RefCell<Vec<String>>,
}
impl Prompt for ScriptedPrompt {
    fn choose(&self, _: &str, _: &[&str]) -> Result<usize, CliError> {
        self.choices.borrow_mut().pop().ok_or(CliError::Cancelled)
    }
    fn text(&self, _: &str) -> Result<String, CliError> {
        self.texts.borrow_mut().pop().ok_or(CliError::Cancelled)
    }
}

pub(crate) type Save = Box<dyn Fn(&str) -> String>;

pub(crate) struct ScriptedEditor(RefCell<Vec<Save>>, tempfile::TempDir);
impl ScriptedEditor {
    pub(crate) fn saving(saves: Vec<Save>) -> ScriptedEditor {
        ScriptedEditor(RefCell::new(saves), tempfile::tempdir().unwrap())
    }
    pub(crate) fn saving_unchanged() -> ScriptedEditor {
        ScriptedEditor::saving(vec![])
    }
}
impl EditorSession for ScriptedEditor {
    fn edit(&self, opened: &str) -> Result<String, EditorError> {
        let mut saves = self.0.borrow_mut();
        Ok(if saves.is_empty() {
            opened.to_string()
        } else {
            saves.remove(0)(opened)
        })
    }
    fn keep(&self, saved: &str) -> Result<std::path::PathBuf, EditorError> {
        let path = self.1.path().join("kept.toml");
        std::fs::write(&path, saved).map_err(|e| EditorError(e.to_string()))?;
        Ok(path)
    }
}
