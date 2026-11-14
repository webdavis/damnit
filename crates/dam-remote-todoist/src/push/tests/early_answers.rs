use std::cell::RefCell;

use dam_protocol::Verdict;

use super::failures::three_subject_updates;
use super::fixtures::sync_body;
use crate::api::fake::{FakeTodoist, rate_limited};
use crate::push::push;

#[test]
fn each_answer_is_handed_out_as_soon_as_todoist_gives_it() {
    let api = FakeTodoist::new(sync_body()).failing_write(2, rate_limited(42));
    let early = RefCell::new(Vec::new());
    let response = push(&api, three_subject_updates(), &mut |result| {
        let writes_so_far = api.recorded.borrow().writes;
        early.borrow_mut().push((result.clone(), writes_so_far));
    })
    .unwrap();
    let early = early.into_inner();
    let writes: Vec<_> = early.iter().map(|(_, writes)| *writes).collect();
    assert_eq!(
        writes,
        vec![1, 2, 2],
        "the first answer came before the second write"
    );
    assert!(matches!(early[0].0.verdict, Verdict::Accepted { .. }));
    let handed_out: Vec<_> = early.into_iter().map(|(result, _)| result).collect();
    assert_eq!(handed_out, response.results);
}
