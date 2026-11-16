use std::process::Stdio;

use crate::support::sandbox::Sandbox;

#[test]
fn ids_piped_from_dam_list_into_dam_done_complete_those_tasks() {
    let sb = Sandbox::new();
    sb.new_object(&["buy milk", "--labels", "quick"]);
    sb.new_object(&["fix sink", "--labels", "quick"]);
    let kept = sb.new_object(&["write report"]);
    let script = "dam list --labels quick | dam done -";
    let out = sb.shell(script).stdin(Stdio::null()).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("2 of 2"),
        "{out:?}"
    );
    let (_, open, _) = sb.dam(&["list"]);
    assert_eq!(open.lines().count(), 1);
    assert!(open.starts_with(&kept[..7]), "{open}");
}

#[test]
fn an_empty_list_piped_into_dam_done_changes_nothing_and_says_so() {
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    let script = "dam list --labels quick | dam done -";
    let out = sb.shell(script).stdin(Stdio::null()).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "dam: no objects match\ndam: no ids came in on standard input; nothing changed\n"
    );
    assert_eq!(sb.dam(&["list"]).1.lines().count(), 1);
}
