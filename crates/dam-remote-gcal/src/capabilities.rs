use dam_protocol::{Capabilities, PROTOCOL_VERSION};

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
mod tests;
