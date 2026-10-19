use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct EditorError(pub(crate) String);

pub(crate) trait EditorSession {
    fn edit(&self, text: &str) -> Result<String, EditorError>;
    fn keep(&self, saved: &str) -> Result<PathBuf, EditorError>;
}

pub(crate) struct EnvEditor {
    command: Vec<String>,
}

impl EnvEditor {
    pub(crate) fn from_env() -> EnvEditor {
        let visual = std::env::var("VISUAL").ok();
        let editor = std::env::var("EDITOR").ok();
        EnvEditor::with_command(choose_command(visual, editor))
    }

    pub(crate) fn with_command(command: Vec<String>) -> EnvEditor {
        EnvEditor { command }
    }
}

const EDITOR_WHEN_NEITHER_VARIABLE_IS_SET: &str = "vi";

fn choose_command(visual: Option<String>, editor: Option<String>) -> Vec<String> {
    let raw = visual
        .filter(|v| !v.trim().is_empty())
        .or_else(|| editor.filter(|v| !v.trim().is_empty()))
        .unwrap_or_else(|| EDITOR_WHEN_NEITHER_VARIABLE_IS_SET.into());
    split_into_arguments_unless_the_whole_names_a_file(&raw)
}

fn split_into_arguments_unless_the_whole_names_a_file(raw: &str) -> Vec<String> {
    let whole = raw.trim();
    if whole.contains(char::is_whitespace) && std::path::Path::new(whole).is_file() {
        return vec![whole.to_string()];
    }
    raw.split_whitespace().map(str::to_string).collect()
}

impl EditorSession for EnvEditor {
    fn edit(&self, text: &str) -> Result<String, EditorError> {
        let (program, args) = self
            .command
            .split_first()
            .ok_or_else(|| EditorError("no editor configured; set VISUAL or EDITOR".into()))?;
        let file = file_holding(text)?;
        let status = Command::new(program)
            .args(args)
            .arg(file.path())
            .status()
            .map_err(|e| EditorError(format!("{program}: {e}")))?;
        if !status.success() {
            return Err(EditorError(format!("{program} exited {status}")));
        }
        std::fs::read_to_string(file.path()).map_err(|e| EditorError(e.to_string()))
    }

    fn keep(&self, saved: &str) -> Result<PathBuf, EditorError> {
        file_holding(saved)?
            .keep()
            .map(|(_, path)| path)
            .map_err(|e| EditorError(e.to_string()))
    }
}

fn file_holding(text: &str) -> Result<tempfile::NamedTempFile, EditorError> {
    let mut file = tempfile::Builder::new()
        .prefix("dam-edit-")
        .suffix(".toml")
        .tempfile()
        .map_err(|e| EditorError(e.to_string()))?;
    file.write_all(text.as_bytes())
        .map_err(|e| EditorError(e.to_string()))?;
    file.flush().map_err(|e| EditorError(e.to_string()))?;
    Ok(file)
}

#[cfg(test)]
mod tests;
