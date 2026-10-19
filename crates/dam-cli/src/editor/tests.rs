use super::*;

#[test]
fn an_editor_installed_at_a_path_with_a_space_is_run_as_one_word() {
    let dir = tempfile::tempdir().unwrap();
    let installed = dir.path().join("My Editor");
    std::fs::write(&installed, "#!/bin/sh\nexit 0\n").unwrap();
    let whole = installed.to_string_lossy().into_owned();
    assert_eq!(choose_command(Some(whole.clone()), None), vec![whole]);
}

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
