use dam_application::{Project, ProjectSettings, TaskTemplate};

use super::super::dispatch_in;
use super::super::tests::new_args;
use super::{global, task};
use crate::args::Command;
use crate::output::Report;
use crate::project_scope::Standing;
use crate::testing::context;

fn checked_out(head: &str) -> tempfile::TempDir {
    let clone = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(clone.path().join(".git")).unwrap();
    std::fs::write(clone.path().join(".git/HEAD"), head).unwrap();
    clone
}

fn labelling_branches(clone: &tempfile::TempDir, defaults: Option<Vec<String>>) -> Standing {
    Standing::Inside(Box::new(Project {
        folder: clone.path().into(),
        settings: ProjectSettings {
            path: "app".into(),
            branch_label: true,
            new_task: TaskTemplate {
                labels: defaults,
                ..TaskTemplate::default()
            },
            ..ProjectSettings::default()
        },
    }))
}

fn labels_after(
    head: &str,
    command: Command,
    defaults: Option<Vec<String>>,
) -> (Vec<String>, Report) {
    let ctx = context();
    let clone = checked_out(head);
    let made = dispatch_in(&ctx, command, labelling_branches(&clone, defaults)).unwrap();
    (task(&ctx).base.labels.into_iter().collect(), made)
}

fn new(labels: &[&str]) -> Command {
    let mut args = new_args("crash", "");
    args.labels = labels.iter().map(|l| l.to_string()).collect();
    Command::New(args)
}

#[test]
fn a_task_made_inside_the_project_is_labelled_with_the_branch() {
    let (labels, made) = labels_after("ref: refs/heads/fix-login\n", new(&[]), None);
    assert_eq!(labels, ["fix-login"]);
    assert!(made.standard_error_lines.is_empty());
}

#[test]
fn the_branch_joins_the_projects_default_labels() {
    let defaults = Some(vec!["app".into()]);
    let (labels, _) = labels_after("ref: refs/heads/fix-login\n", new(&[]), defaults);
    assert_eq!(labels, ["app", "fix-login"]);
}

#[test]
fn labels_given_as_flags_keep_the_branch_label() {
    let defaults = Some(vec!["app".into()]);
    let (labels, _) = labels_after("ref: refs/heads/fix-login\n", new(&["mine"]), defaults);
    assert_eq!(labels, ["fix-login", "mine"]);
}

#[test]
fn a_detached_head_adds_no_label_and_no_warning() {
    let head = "3f786850e387550fdab836ed7e6dc881de23001b\n";
    let (labels, made) = labels_after(head, new(&[]), None);
    assert!(labels.is_empty());
    assert!(made.standard_error_lines.is_empty());
}

#[test]
fn a_branch_that_is_no_label_adds_none_and_warns_without_quoting_it() {
    for head in [
        "ref: refs/heads/bad name\n",
        "ref: refs/heads/evil\u{1b}[31m\n",
        "ref: refs/heads/a|b\n",
    ] {
        let (labels, made) = labels_after(head, new(&[]), None);
        assert!(labels.is_empty(), "{head:?}");
        assert_eq!(
            made.standard_error_lines,
            [
                "dam: warning: the checked-out git branch's name is not a label dam can use, so the task has no branch label"
            ],
            "{head:?}"
        );
    }
}

#[test]
fn with_global_no_branch_label_is_added() {
    let ctx = context();
    let clone = checked_out("ref: refs/heads/fix-login\n");
    dispatch_in(
        &ctx,
        global(new_args("crash", "")),
        labelling_branches(&clone, None),
    )
    .unwrap();
    assert!(task(&ctx).base.labels.is_empty());
}
