use dam_application::{PullReport, PulledHookRun, PushReport, RemoteName};
use dam_domain::{CommitId, Oid};

use super::pull_view::{PullRow, Pulled};
use super::view::{PushRow, Pushed};
use crate::view::{self, Human, View};

fn lines(human: Human) -> Vec<String> {
    let Human::Lines(lines) = human else {
        panic!("expected plain lines")
    };
    lines
}

fn pushed_with(held: Option<usize>) -> PushReport {
    PushReport {
        remote: RemoteName("fake".into()),
        sent: 1,
        succeeded: 1,
        failed: vec![],
        skipped: 0,
        held,
    }
}

fn pulled_with(task_pulled: PulledHookRun) -> PullReport {
    PullReport {
        remote: RemoteName("fake".into()),
        created: 1,
        updated: 0,
        conflicts: 0,
        removed_upstream: 0,
        unchanged: 0,
        creates_matched: 0,
        values_kept: 0,
        deletes_confirmed: 0,
        task_pulled: Some(task_pulled),
    }
}

#[test]
fn a_push_that_held_an_object_for_a_conflict_says_where_to_settle_it() {
    let report = pushed_with(Some(1));
    let shown = Pushed {
        remotes: vec![PushRow::new(&report)],
    };
    assert_eq!(
        lines(shown.human()),
        ["fake: 1 sent, 1 ok, 0 failed, 0 skipped\n  \
          1 held for a conflict; see dam status, settle with dam resolve"]
    );
}

#[test]
fn a_push_answer_hidden_from_a_token_without_conflict_read_has_no_held_count() {
    let report = pushed_with(None);
    let row = serde_json::to_value(PushRow::new(&report)).unwrap();
    assert_eq!(row["held"], serde_json::Value::Null);
}

#[test]
fn a_pull_says_what_its_pulled_hook_did() {
    let report = pulled_with(PulledHookRun {
        ran: 1,
        failed: 1,
        ..PulledHookRun::default()
    });
    let shown = Pulled {
        remotes: vec![PullRow::new(&report)],
    };
    assert_eq!(
        view::report(&shown).unwrap().human,
        "pull ── what each remote sent\n  ✓ fake: 1 new\n    \
         ⚠ task_pulled: 1 ran, 0 changed, 1 failed, 0 not reached"
    );
}

#[test]
fn a_pull_names_the_commit_its_pulled_hook_made_in_text_and_json() {
    let commit = CommitId::generate(&mut |x: &mut [u8]| x.fill(7));
    let report = pulled_with(PulledHookRun {
        ran: 1,
        changed: 1,
        commit: Some(commit.clone()),
        ..PulledHookRun::default()
    });
    let shown = Pulled {
        remotes: vec![PullRow::new(&report)],
    };
    assert_eq!(
        view::report(&shown).unwrap().human.lines().last().unwrap(),
        format!(
            "    ✓ task_pulled: 1 ran, 1 changed, 0 failed, 0 not reached, commit {}",
            commit.short()
        )
    );
    assert_eq!(
        serde_json::to_value(&shown).unwrap()["remotes"][0]["task_pulled"],
        serde_json::json!({"ran": 1, "changed": 1, "failed": 0, "not_reached": 0,
                           "commit": commit.to_string()})
    );
}

#[test]
fn a_push_hidden_held_count_still_says_some_are_held() {
    let report = pushed_with(None);
    let shown = Pushed {
        remotes: vec![PushRow::new(&report)],
    };
    assert_eq!(
        lines(shown.human()),
        ["fake: 1 sent, 1 ok, 0 failed, 0 skipped\n  \
          some held for a conflict; see dam status, settle with dam resolve"]
    );
}

#[test]
fn a_push_names_each_object_that_failed_and_why_in_text_and_json() {
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(3));
    let report = PushReport {
        failed: vec![(oid.clone(), "the remote said no".into())],
        ..pushed_with(Some(0))
    };
    let shown = Pushed {
        remotes: vec![PushRow::new(&report)],
    };
    assert_eq!(
        lines(shown.human()),
        [format!(
            "fake: 1 sent, 1 ok, 1 failed, 0 skipped\n  {}: the remote said no",
            oid.short()
        )]
    );
    assert_eq!(
        serde_json::to_value(&shown).unwrap()["remotes"][0]["failed"],
        serde_json::json!([{"oid": oid.to_string(), "why": "the remote said no"}])
    );
}

#[test]
fn a_push_with_no_remotes_says_so() {
    let pushed = Pushed { remotes: vec![] };
    assert_eq!(lines(pushed.human()), ["no remotes to push to"]);
}
