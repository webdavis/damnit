use super::*;

#[test]
fn credential_variables_are_upper_snake() {
    assert_eq!(
        credential_variable("todoist", "api_token"),
        "DAM_TODOIST_API_TOKEN"
    );
    assert_eq!(
        credential_variable("my-cal", "client-id"),
        "DAM_MY_CAL_CLIENT_ID"
    );
}
