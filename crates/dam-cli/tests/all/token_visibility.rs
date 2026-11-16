use crate::support;

use dam_domain::Token;
use support::sandbox::Sandbox;
use support::token_table::{Grant, TokenTable};

fn work_only_reader(sb: &Sandbox) -> String {
    let held = Token::generate(&mut |b: &mut [u8]| b.fill(2)).expose();
    sb.add_token(
        "reader",
        TokenTable::valued(&held).grant(
            Grant::allowing(&[
                "task:read",
                "event:read",
                "working:read",
                "stage:read",
                "history:read",
            ])
            .paths(&["work"]),
        ),
    );
    held
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
    let (_, owner_view, _) = sb.dam(&["show", &report, "-o", "json"]);
    assert!(owner_view.contains(&diary) && owner_view.contains(&dentist));

    for format in [vec!["-o", "json"], vec!["-o", "toon"], vec![]] {
        for read in [vec!["ls"], vec!["show", &report]] {
            let args = [read, format.clone()].concat();
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
