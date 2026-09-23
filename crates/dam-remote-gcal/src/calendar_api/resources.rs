//! Google's event resource, as much of it as dam reads. Every field but an
//! event's id may be missing: a cancelled item may carry nothing else.

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EventsPage {
    pub summary: Option<String>,
    pub time_zone: Option<String>,
    pub items: Vec<GoogleEvent>,
    pub next_page_token: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEvent {
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub start: Option<GoogleTime>,
    #[serde(default)]
    pub end: Option<GoogleTime>,
    #[serde(default)]
    pub transparency: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub color_id: Option<String>,
    #[serde(default)]
    pub organizer: Option<GooglePerson>,
    #[serde(default)]
    pub attendees: Vec<GoogleAttendee>,
    #[serde(default)]
    pub conference_data: Option<ConferenceData>,
    #[serde(default)]
    pub attachments: Vec<GoogleAttachment>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GoogleTime {
    pub date: Option<String>,
    pub date_time: Option<String>,
    pub time_zone: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GooglePerson {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GoogleAttendee {
    pub email: Option<String>,
    pub response_status: Option<String>,
    /// Whether this attendee is the calendar this copy of the event is on.
    #[serde(rename = "self")]
    pub is_self: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConferenceData {
    pub conference_solution: Option<ConferenceSolution>,
    pub entry_points: Vec<EntryPoint>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConferenceSolution {
    pub name: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EntryPoint {
    pub entry_point_type: Option<String>,
    pub uri: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GoogleAttachment {
    pub file_url: Option<String>,
    pub title: Option<String>,
    pub mime_type: Option<String>,
}
