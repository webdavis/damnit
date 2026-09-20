use dam_protocol::{Capabilities, PROTOCOL_VERSION};

pub fn capabilities() -> Capabilities {
    Capabilities {
        protocol: PROTOCOL_VERSION,
        kinds: vec!["task".into()],
        fields: [
            "subject",
            "body",
            "path",
            "labels",
            "done",
            "priority",
            "due",
            "deadline",
            "recurrence",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
        credentials: vec!["api_token".into()],
        incremental: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_todoist_helper_scope() {
        let caps = capabilities();
        assert_eq!(caps.protocol, PROTOCOL_VERSION);
        assert!(caps.supported());
        assert_eq!(caps.kinds, vec!["task".to_string()]);
        assert_eq!(caps.credentials, vec!["api_token".to_string()]);
        assert!(!caps.incremental);
        assert!(caps.fields.contains(&"deadline".to_string()));
    }
}
