use super::*;

#[test]
fn matches_the_todoist_helper_scope() {
    let caps = capabilities();
    assert_eq!(caps.protocol, PROTOCOL_VERSION);
    assert!(caps.supported());
    assert_eq!(caps.kinds, vec!["task".to_string()]);
    assert_eq!(caps.credentials, vec!["api_token".to_string()]);
    assert!(caps.incremental);
    assert!(caps.fields.contains(&"deadline".to_string()));
}
