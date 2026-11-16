use crate::select::{NameFault, filter_name_fault};

#[test]
fn a_plain_name_is_kept() {
    for name in ["today", "standup", "my-week", "q4", "th"] {
        assert_eq!(filter_name_fault(name), None, "{name}");
    }
}

#[test]
fn a_name_a_value_already_means_is_refused() {
    assert_eq!(filter_name_fault(""), Some(NameFault::Empty));
    assert_eq!(filter_name_fault("none"), Some(NameFault::None));
    for name in ["thu", "monday", "1tue", "-1fri", "15", "-1"] {
        assert_eq!(filter_name_fault(name), Some(NameFault::OnDay), "{name}");
    }
    for (name, start) in [
        ("not:x", "not:"),
        ("=x", "="),
        ("in:x", "in:"),
        ("on:x", "on:"),
    ] {
        assert_eq!(filter_name_fault(name), Some(NameFault::Starts(start)));
    }
    assert_eq!(filter_name_fault("a,b"), Some(NameFault::Comma));
}
