use dam_application::{Blockers, Refusal};
use dam_domain::{LabelViolation, Oid, Tracked};

use super::oid;

fn abab000() -> String {
    Oid::generate(&mut |x: &mut [u8]| {
        x.fill(0);
        x[..2].fill(0xab);
    })
    .to_string()
}

pub(super) fn every_refusal_with_its_text() -> Vec<(Refusal, &'static str)> {
    let mut samples = refusals_by_object_rules();
    samples.extend(super::access_samples::access_refusals_with_their_text());
    samples.push((
        Refusal::SubjectNamesNoFolder(oid(1)),
        "0101010 has objects in its folder, so its subject cannot be empty, . or ..",
    ));
    samples
}

fn refusals_by_object_rules() -> Vec<(Refusal, &'static str)> {
    vec![
        (
            Refusal::Blocked {
                oid: oid(1),
                blockers: Blockers {
                    children: vec![oid(2)],
                    ..Blockers::default()
                },
            },
            "0101010 has 1 open child: 0202020; pass --force to complete it anyway",
        ),
        (
            Refusal::Cycle {
                oid: oid(1),
                path: vec![oid(2), oid(1)],
            },
            "0101010 cannot depend on that: it would form a cycle through 0202020 -> 0101010",
        ),
        (
            Refusal::Labels(LabelViolation::Exclusive {
                category: "effort".into(),
                held: vec!["deep".into(), "shallow".into()],
            }),
            "category \"effort\" allows one value, got deep, shallow",
        ),
        (
            Refusal::UnknownCategory("effort".into()),
            "\"effort\" is not a declared category",
        ),
        (
            Refusal::NotACategoryValue {
                category: "effort".into(),
                value: "home".into(),
            },
            "home is not a value of category effort",
        ),
        (
            Refusal::NoSuchObject("abab".into()),
            "no object matches \"abab\"",
        ),
        (
            Refusal::NoWorkingObject("abab".into()),
            "no object in the working layer matches \"abab\"; one removed from it is named by its full oid",
        ),
        (
            Refusal::AmbiguousPrefix {
                text: "abab".into(),
                matches: vec![oid(0xab).to_string(), abab000()],
            },
            "abab matches more than one id: abababa, abab000; give more characters",
        ),
        (
            Refusal::NoSuchRemote("todoist".into()),
            "no remote named \"todoist\"",
        ),
        (
            Refusal::RemoteExists("todoist".into()),
            "remote todoist already exists",
        ),
        (
            Refusal::NotATask(oid(1)),
            "0101010 is an event; events are not completed",
        ),
        (
            Refusal::NotAnEvent(oid(1)),
            "0101010 is a task; start, end and location are event fields",
        ),
        (
            Refusal::NotCompleted(oid(1)),
            "0101010 is not completed, so there is nothing to reopen",
        ),
        (
            Refusal::NotCommitted(oid(1)),
            "0101010 has no commit behind it; dam rm removes it instead",
        ),
        (
            Refusal::DirtyOnPull { oid: oid(1) },
            "0101010 changed upstream and has uncommitted local changes; commit or reset it, then pull again",
        ),
        (
            Refusal::MoveInsideItself(oid(1)),
            "0101010 cannot move inside itself",
        ),
        (Refusal::NothingToCommit, "nothing to commit"),
        (
            Refusal::NeedsAnAnswer,
            "a question needs an answer; drop --json/--toon to answer interactively",
        ),
        (
            Refusal::NeedsAnEditor,
            "-e opens an editor; drop --json/--toon to use it",
        ),
        (
            Refusal::UnresolvedConflicts(Some(2)),
            "2 conflicts are unresolved; run dam resolve",
        ),
        (
            Refusal::UnresolvedConflicts(None),
            "conflicts are unresolved; run dam resolve",
        ),
        (
            Refusal::MissingCredential {
                remote: "todoist".into(),
                name: "api_token".into(),
            },
            "remote \"todoist\" needs api_token; set api_token, api_token_command or api_token_env in its config",
        ),
        (
            Refusal::StaleRemote {
                remote: "gcal".into(),
                age: None,
                limit: 3600,
            },
            "remote \"gcal\" has never been pulled; run dam pull gcal",
        ),
        (
            Refusal::StaleRemote {
                remote: "gcal".into(),
                age: Some(7200),
                limit: 3600,
            },
            "remote \"gcal\" last pulled 7200s ago, longer than --max-age allows (3600s); run dam pull gcal",
        ),
        (
            Refusal::UnreadableRecurrence {
                rule: "every other week".into(),
                tracked: Tracked::No,
            },
            "cannot roll \"every other week\" forward; set --recurrence",
        ),
        (
            Refusal::UnreadableRecurrence {
                rule: "every other week".into(),
                tracked: Tracked::ByRemote,
            },
            "cannot roll \"every other week\" forward; complete it in Todoist or set --recurrence",
        ),
        (
            Refusal::NoConflict(oid(1)),
            "0101010 has no conflict to resolve",
        ),
    ]
}
