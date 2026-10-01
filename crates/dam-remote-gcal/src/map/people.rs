use dam_protocol::{WireAttachment, WireAttendee, WireConference, WirePerson};

use super::words::response;
use crate::calendar_api::GoogleEvent;

pub(super) fn attendees(event: &GoogleEvent) -> Vec<WireAttendee> {
    event
        .attendees
        .iter()
        .filter_map(|a| {
            Some(WireAttendee {
                email: a.email.clone()?,
                response: response(a.response_status.as_deref()).into(),
                is_self: a.is_self,
            })
        })
        .collect()
}

pub(super) fn organizer(event: &GoogleEvent) -> Option<WirePerson> {
    let o = event.organizer.as_ref()?;
    Some(WirePerson {
        email: o.email.clone()?,
        name: o.display_name.clone(),
    })
}

pub(super) fn conference(event: &GoogleEvent) -> Option<WireConference> {
    let c = event.conference_data.as_ref()?;
    let video = c
        .entry_points
        .iter()
        .find(|p| p.entry_point_type.as_deref() == Some("video"))?;
    Some(WireConference {
        provider: c
            .conference_solution
            .as_ref()
            .and_then(|s| s.name.clone())
            .unwrap_or_default(),
        url: video.uri.clone()?,
    })
}

pub(super) fn attachments(event: &GoogleEvent) -> Vec<WireAttachment> {
    event
        .attachments
        .iter()
        .filter_map(|a| {
            Some(WireAttachment {
                url: a.file_url.clone()?,
                title: a.title.clone().unwrap_or_default(),
                mime_type: a.mime_type.clone(),
            })
        })
        .collect()
}
