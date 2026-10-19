mod resources;

pub use resources::{
    ConferenceData, ConferenceSolution, EntryPoint, EventsPage, GoogleAttachment, GoogleAttendee,
    GoogleEvent, GooglePerson, GoogleTime,
};

use crate::api_error::{ApiError, SERVICE};
use crate::encoding::{form, percent_encoded};
use crate::{Endpoints, Secret};
use dam_helper::{ServiceError, read_answer};

pub const PAGE_SIZE: u32 = 2500;
pub const MAX_PAGES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub from: jiff::Timestamp,
    pub to: jiff::Timestamp,
}

#[derive(Debug)]
pub struct Listing {
    pub calendar: String,
    pub summary: Option<String>,
    pub time_zone: Option<String>,
    pub items: Vec<GoogleEvent>,
}

pub trait Calendar {
    fn events(&self, calendar: &str, window: &Window) -> Result<Listing, ApiError>;
}

pub struct CalendarApi {
    agent: ureq::Agent,
    base: String,
    access: Secret,
}

impl CalendarApi {
    pub fn new(agent: ureq::Agent, endpoints: &Endpoints, access: Secret) -> CalendarApi {
        CalendarApi {
            agent,
            base: endpoints.calendar.clone(),
            access,
        }
    }

    fn page(&self, url: &str) -> Result<EventsPage, ApiError> {
        let response = self
            .agent
            .get(url)
            .header("authorization", &format!("Bearer {}", self.access.expose()))
            .call()
            .map_err(|e| ServiceError::transport(SERVICE, e))?;
        let body = read_answer(SERVICE, response)?;
        serde_json::from_str(&body).map_err(|e| ServiceError::decode(SERVICE, e).into())
    }
}

impl Calendar for CalendarApi {
    fn events(&self, calendar: &str, window: &Window) -> Result<Listing, ApiError> {
        let url = format!(
            "{}/calendars/{}/events",
            self.base,
            percent_encoded(calendar)
        );
        let (from, to) = (window.from.to_string(), window.to.to_string());
        let size = PAGE_SIZE.to_string();
        let mut listing = Listing {
            calendar: calendar.to_string(),
            summary: None,
            time_zone: None,
            items: Vec::new(),
        };
        let mut next: Option<String> = None;
        for _ in 0..MAX_PAGES {
            let mut query = vec![
                ("singleEvents", "true"),
                ("showDeleted", "true"),
                ("maxResults", size.as_str()),
                ("timeMin", from.as_str()),
                ("timeMax", to.as_str()),
            ];
            if let Some(token) = &next {
                query.push(("pageToken", token.as_str()));
            }
            let page = self.page(&format!("{url}?{}", form(&query)))?;
            listing.summary = listing.summary.or(page.summary);
            listing.time_zone = listing.time_zone.or(page.time_zone);
            listing.items.extend(page.items);
            match page.next_page_token {
                Some(token) => next = Some(token),
                None => return Ok(listing),
            }
        }
        Err(ApiError::TooManyPages { limit: MAX_PAGES })
    }
}

#[cfg(test)]
pub(crate) mod fake;
