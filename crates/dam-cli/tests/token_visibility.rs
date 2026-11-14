mod support;

use dam_domain::Token;
use support::errors::error_document;
use support::sandbox::Sandbox;

#[test]
fn a_token_reads_only_what_its_grants_cover_and_nothing_hints_at_the_rest() {
    let _guard =
        support::guard("a_token_reads_only_what_its_grants_cover_and_nothing_hints_at_the_rest");
    let sb = Sandbox::new();
    let held = Token::generate(&mut |b: &mut [u8]| b.fill(1)).expose();
    sb.add_config(&format!(
        "[token.tool]\nvalue = \"{held}\"\n[[token.tool.grant]]\nallow = [\"task:read\"]\npaths = [\"work\"]\n"
    ));
    sb.new_object(&["report", "--path", "work"]);
    let hidden = sb.new_object(&["diary", "--path", "home"]);

    let out = sb.output_holding(&held, &["ls", "--json", "--no-pull"]);
    assert!(out.status.success(), "{out:?}");
    let listed = String::from_utf8_lossy(&out.stdout);
    assert!(
        listed.contains("report") && !listed.contains("diary"),
        "{listed}"
    );

    let out = sb.output_holding(&held, &["show", &hidden, "--json", "--no-pull"]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(error_document(&out)["error"]["rule"], "no_such_object");
}

fn work_only_reader(sb: &Sandbox) -> String {
    let held = Token::generate(&mut |b: &mut [u8]| b.fill(2)).expose();
    sb.add_config(&format!(
        "[token.reader]\nvalue = \"{held}\"\n[[token.reader.grant]]\n\
         allow = [\"task:read\", \"event:read\", \"working:read\", \"stage:read\", \"history:read\"]\n\
         paths = [\"work\"]\n"
    ));
    held
}

#[test]
fn a_path_restricted_token_sees_no_event_outside_its_path_in_the_agenda() {
    let _guard =
        support::guard("a_path_restricted_token_sees_no_event_outside_its_path_in_the_agenda");
    let sb = Sandbox::new();
    let held = work_only_reader(&sb);
    for (subject, path) in [("standup", "work"), ("dentist", "home")] {
        sb.new_object(&[
            "--event",
            subject,
            "--path",
            path,
            "--start",
            "2026-09-25T14:00",
            "--end",
            "2026-09-25T14:30",
        ]);
    }
    let window = [
        "agenda",
        "--from",
        "2026-09-25",
        "--to",
        "2026-09-26",
        "--json",
        "--no-pull",
    ];
    let out = sb.output_holding(&held, &window);
    assert!(out.status.success(), "{out:?}");
    let listed = String::from_utf8_lossy(&out.stdout);
    assert!(
        listed.contains("standup") && !listed.contains("dentist"),
        "{listed}"
    );
}

#[test]
fn a_path_restricted_token_is_refused_the_reads_that_cannot_filter_by_path_yet() {
    let _guard = support::guard(
        "a_path_restricted_token_is_refused_the_reads_that_cannot_filter_by_path_yet",
    );
    let sb = Sandbox::new();
    let held = work_only_reader(&sb);
    sb.new_object(&["diary", "--path", "home"]);
    let status = sb.output_holding(&held, &["status", "--json"]);
    assert!(status.status.success(), "{status:?}");
    assert!(!String::from_utf8_lossy(&status.stdout).contains("diary"));
    for read in ["diff", "log"] {
        let out = sb.output_holding(&held, &[read, "--json"]);
        assert_eq!(out.status.code(), Some(4), "{read}");
        assert_eq!(
            error_document(&out)["error"]["rule"],
            "not_permitted",
            "{read}"
        );
        assert!(!String::from_utf8_lossy(&out.stderr).contains("diary"));
    }
}

#[test]
fn a_token_never_sees_the_oid_of_a_hidden_dependency_or_attached_event() {
    let _guard =
        support::guard("a_token_never_sees_the_oid_of_a_hidden_dependency_or_attached_event");
    let sb = Sandbox::new();
    let held = work_only_reader(&sb);
    let diary = sb.new_object(&["diary", "--path", "home"]);
    let dentist = sb.new_object(&[
        "--event",
        "dentist",
        "--path",
        "home",
        "--start",
        "2026-09-25T14:00",
        "--end",
        "2026-09-25T14:30",
    ]);
    let report = sb.new_object(&["report", "--path", "work"]);
    let (ok, _, err) = sb.dam(&["edit", &report, "--depends", &diary, "--attach", &dentist]);
    assert!(ok, "{err}");
    let (_, owner_view, _) = sb.dam(&["show", &report, "--json", "--no-pull"]);
    assert!(owner_view.contains(&diary) && owner_view.contains(&dentist));

    for format in [vec!["--json"], vec!["--toon"], vec![]] {
        for read in [vec!["ls"], vec!["show", &report]] {
            let args = [read, format.clone(), vec!["--no-pull"]].concat();
            let out = sb.output_holding(&held, &args);
            assert!(out.status.success(), "{args:?}: {out:?}");
            let shown = String::from_utf8_lossy(&out.stdout);
            assert!(shown.contains("report"), "{args:?}: {shown}");
            assert!(
                !shown.contains(&diary) && !shown.contains(&dentist),
                "{args:?}: {shown}"
            );
        }
    }
}
