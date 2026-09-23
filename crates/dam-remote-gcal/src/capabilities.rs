use dam_protocol::{Capabilities, PROTOCOL_VERSION};

/// No kinds, so dam pushes nothing here: this remote is read-only. Every
/// field Google's event resource carries that dam models, and none of dam's
/// own, so a pull never overwrites labels, dependencies, reminders or
/// recurrence. Incremental: the helper reads `since` and answers a token.
pub fn capabilities() -> Capabilities {
    let fields = [
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
    ];
    Capabilities {
        protocol: PROTOCOL_VERSION,
        kinds: vec![],
        fields: fields.iter().map(|f| f.to_string()).collect(),
        credentials: vec![
            "client_id".into(),
            "client_secret".into(),
            "refresh_token".into(),
        ],
        incremental: true,
    }
}

#[cfg(test)]
mod tests {
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
}
