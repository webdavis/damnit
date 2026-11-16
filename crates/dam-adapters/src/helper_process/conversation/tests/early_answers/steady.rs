use std::time::{Duration, Instant};

use dam_application::{PushOutcome, RemoteHelper};

use super::super::super::super::testing::install;
use super::super::until_the_helper_has_complained;
use super::{accepted, deletes, early, script_that_says_it_is_up, silent_for_at_most};

#[test]
fn a_helper_answering_steadily_is_not_cut_off_when_the_whole_call_outlasts_the_deadline() {
    let dir = tempfile::tempdir().unwrap();
    let answers = 1..=10;
    let pause_a_tenth_of_the_deadline = "sleep 0.04\n";
    let results: Vec<String> = answers
        .clone()
        .map(|n| {
            format!(
                "{{\"oid\":\"{}\",\"ok\":true,\"remote_id\":\"r{n}\"}}",
                format!("{n:02}").repeat(20)
            )
        })
        .collect();
    let reply = format!("printf '{{\"results\":[{}]}}\\n'\n", results.join(","));
    let steps: Vec<String> = answers
        .clone()
        .map(early)
        .chain([reply])
        .map(|step| format!("{pause_a_tenth_of_the_deadline}{step}"))
        .collect();
    let steps: Vec<&str> = steps.iter().map(String::as_str).collect();
    let launcher = install(dir.path(), &script_that_says_it_is_up(&steps));
    let mut helper = launcher.spawn(&silent_for_at_most(400), &[]).unwrap();
    until_the_helper_has_complained(&helper);
    let started = Instant::now();
    let pushed = helper.push(deletes(answers.clone())).unwrap();
    assert!(
        started.elapsed() > Duration::from_millis(400),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(
        pushed,
        PushOutcome {
            outcomes: answers.map(accepted).collect(),
            stopped_by: None,
        }
    );
}
