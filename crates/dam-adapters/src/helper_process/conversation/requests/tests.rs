use dam_application::HelperError;
use dam_protocol::PushResponse;
use serde_json::json;

use super::reply_from;

#[test]
fn a_push_answered_with_an_array_is_refused_as_not_a_json_object() {
    let refused = reply_from::<PushResponse>(json!([[]])).unwrap_err();
    assert!(
        matches!(&refused, HelperError::Protocol(why) if why == "expected a JSON object"),
        "{refused:?}"
    );
}
