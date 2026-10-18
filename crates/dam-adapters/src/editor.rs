use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use dam_application::{EditorError, EditorSession};

pub struct EnvEditor {
    command: Vec<String>,
}

impl EnvEditor {
    pub fn from_env() -> EnvEditor {
        let visual = std::env::var("VISUAL").ok();
        let editor = std::env::var("EDITOR").ok();
        EnvEditor::with_command(choose_command(visual, editor))
    }

    pub fn with_command(command: Vec<String>) -> EnvEditor {
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
mod tests {
    #[test]
    fn an_editor_installed_at_a_path_with_a_space_is_run_as_one_word() {
        let dir = tempfile::tempdir().unwrap();
        let installed = dir.path().join("My Editor");
        std::fs::write(&installed, "#!/bin/sh\nexit 0\n").unwrap();
        let whole = installed.to_string_lossy().into_owned();
        assert_eq!(
            super::choose_command(Some(whole.clone()), None),
            vec![whole]
        );
    }

    use super::*;
    use dam_application::EditorSession;

    #[test]
    fn the_editor_sees_the_text_and_its_save_comes_back() {
        let editor = EnvEditor::with_command(vec![
            "sh".into(),
            "-c".into(),
            "printf 'edited\\n' >> \"$1\"".into(),
            "dam-test".into(),
        ]);
        assert_eq!(editor.edit("original\n").unwrap(), "original\nedited\n");
    }

    #[test]
    fn a_kept_save_stays_on_disk_with_its_text() {
        let path = EnvEditor::with_command(vec![]).keep("typed\n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "typed\n");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn a_failing_editor_is_an_error() {
        let editor = EnvEditor::with_command(vec!["sh".into(), "-c".into(), "exit 7".into()]);
        assert!(editor.edit("x").is_err());
    }

    #[test]
    fn an_empty_command_is_an_error() {
        assert!(EnvEditor::with_command(vec![]).edit("x").is_err());
    }

    #[test]
    fn visual_wins_over_editor() {
        assert_eq!(
            choose_command(Some("emacs".into()), Some("vim".into())),
            vec!["emacs".to_string()]
        );
    }

    #[test]
    fn a_blank_visual_falls_through_to_editor() {
        assert_eq!(
            choose_command(Some("  ".into()), Some("vim".into())),
            vec!["vim".to_string()]
        );
    }

    #[test]
    fn neither_set_falls_back_to_vi() {
        assert_eq!(choose_command(None, None), vec!["vi".to_string()]);
    }

    #[test]
    fn a_multi_word_value_splits_on_whitespace() {
        assert_eq!(
            choose_command(Some("code --wait".into()), None),
            vec!["code".to_string(), "--wait".to_string()]
        );
    }
}
