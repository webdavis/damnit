//! Calendar API v3 `events.list` over one calendar and one window.

mod resources;

pub use resources::{
    ConferenceData, ConferenceSolution, EntryPoint, EventsPage, GoogleAttachment, GoogleAttendee,
    GoogleEvent, GooglePerson, GoogleTime,
};

use crate::api_error::{ApiError, bounded};
use crate::encoding::{form, percent_encoded};
use crate::http::{MAX_ANSWER, ReadError, read};
use crate::{Endpoints, Secret};

/// The most `events.list` answers in one page.
pub const PAGE_SIZE: u32 = 2500;
/// How many pages one calendar may take. At the page size above that is
/// 160,000 events, far past a pull's line, and it stops a server that names a
/// next page forever.
pub const MAX_PAGES: usize = 64;

const TOO_MANY_REQUESTS: u16 = 429;

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

    /// Every event whose end is after `window.from` and whose start is
    /// before `window.to`, recurring events expanded into instances, cancelled
    /// ones included, over as many pages as Google names.
    pub fn events(&self, calendar: &str, window: &Window) -> Result<Listing, ApiError> {
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

    fn page(&self, url: &str) -> Result<EventsPage, ApiError> {
        let response = self
            .agent
            .get(url)
            .header("authorization", &format!("Bearer {}", self.access.expose()))
            .call()
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        let answer = read(response, MAX_ANSWER).map_err(|e| match e {
            ReadError::TooLarge { limit } => ApiError::TooLarge { limit },
            ReadError::Transport(s) => ApiError::Transport(s),
        })?;
        if answer.status == TOO_MANY_REQUESTS {
            return Err(ApiError::RateLimited {
                retry_after: answer.retry_after,
                body: bounded(&answer.body),
            });
        }
        if !(200..300).contains(&answer.status) {
            return Err(ApiError::Http {
                status: answer.status,
                body: bounded(&answer.body),
            });
        }
        serde_json::from_str(&answer.body).map_err(|e| ApiError::Decode(e.to_string()))
    }
}
