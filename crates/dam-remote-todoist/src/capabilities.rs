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
        incremental: true,
    }
}

#[cfg(test)]
mod tests;
