use dam_application::Refusal;

use super::oid;

pub(super) fn planning_refusals_with_their_text() -> Vec<(Refusal, &'static str)> {
    vec![(
        Refusal::FocusFull {
            day: jiff::civil::date(2026, 11, 12),
            focused: vec![oid(1), oid(2), oid(3)],
        },
        "2026-11-12 already has 3 focus tasks: 0101010, 0202020, 0303030; clear one with --no-focus first",
    )]
}
