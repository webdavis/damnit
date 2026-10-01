use super::*;

#[test]
fn no_kinds_every_google_field_and_the_three_credentials() {
    let caps = capabilities();
    assert!(caps.supported());
    assert!(caps.kinds.is_empty(), "a declared kind is one dam pushes");
    assert_eq!(
        caps.credentials,
        vec!["client_id", "client_secret", "refresh_token"]
    );
    assert!(caps.incremental);
    for field in [
        "subject",
        "body",
        "path",
        "start",
        "end",
        "timezone",
        "location",
        "attendees",
        "status",
        "transparency",
        "visibility",
        "event_type",
        "color",
        "organizer",
        "conference",
        "attachments",
    ] {
        assert!(caps.fields.iter().any(|f| f == field), "{field}");
    }
    for dam_only in ["labels", "depends", "reminders", "recurrence"] {
        assert!(
            !caps.fields.iter().any(|f| f == dam_only),
            "{dam_only} is dam's own"
        );
    }
}
