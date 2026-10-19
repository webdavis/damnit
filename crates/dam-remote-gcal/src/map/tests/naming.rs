use super::super::*;
use super::{google, listing, live_in};

#[test]
fn a_cancelled_item_is_its_remote_id_whatever_else_it_lacks() {
    let mapped = map_event(
        &listing(),
        &google(serde_json::json!({"id": "gone", "status": "cancelled"})),
    )
    .unwrap();
    assert!(matches!(mapped, Mapped::Cancelled(id) if id == "primary/gone"));
}

#[test]
fn a_calendar_title_is_one_path_segment() {
    let all_day = serde_json::json!({"id": "e", "start": {"date": "2026-09-25"}, "end": {"date": "2026-09-26"}});
    let mut l = listing();
    l.summary = Some("Work/Team".into());
    assert_eq!(live_in(&l, all_day.clone()).0.path, "Work-Team/");
    for nothing in ["  ", ".", ".."] {
        l.summary = Some(nothing.into());
        assert_eq!(
            live_in(&l, all_day.clone()).0.path,
            "primary/",
            "{nothing:?}"
        );
    }
    l.summary = None;
    l.calendar = "team/x@group".into();
    assert_eq!(live_in(&l, all_day).0.path, "team-x@group/");
}
