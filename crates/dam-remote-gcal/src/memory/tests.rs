use std::collections::BTreeSet;

use super::*;

fn memory(entries: &[(&str, i64)]) -> Memory {
    Memory {
        reported: entries
            .iter()
            .map(|(id, end)| (id.to_string(), *end))
            .collect(),
    }
}

#[test]
fn an_absent_or_unreadable_token_is_an_empty_memory() {
    assert_eq!(Memory::read(None), Memory::default());
    assert_eq!(Memory::read(Some("not json")), Memory::default());
    let m = memory(&[("primary/a", 10)]);
    assert_eq!(Memory::read(Some(&m.token())), m);
}

#[test]
fn an_unseen_id_still_in_the_window_has_vanished_whichever_calendar_it_came_from() {
    let m = memory(&[
        ("primary/seen", 500),
        ("primary/moved", 500),
        ("primary/aged", 100),
        ("primary/edge", 200),
        ("dropped@x/c", 500),
    ]);
    let seen: BTreeSet<String> = ["primary/seen".to_string()].into();
    assert_eq!(m.vanished(&seen, 200), vec!["dropped@x/c", "primary/moved"]);
}
