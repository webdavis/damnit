use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use dam_helper::ServiceError;

use super::{Calendar, GoogleEvent, Listing, Window};
use crate::api_error::{ApiError, SERVICE};

#[derive(Default)]
pub(crate) struct FakeCalendar {
    listings: RefCell<HashMap<String, VecDeque<Listing>>>,
    pub(crate) windows_asked: RefCell<Vec<Window>>,
}

impl FakeCalendar {
    pub(crate) fn listing(self, calendar: &str, summary: &str, items: serde_json::Value) -> Self {
        let items: Vec<GoogleEvent> = serde_json::from_value(items).unwrap();
        self.listings
            .borrow_mut()
            .entry(calendar.to_string())
            .or_default()
            .push_back(Listing {
                calendar: calendar.to_string(),
                summary: Some(summary.to_string()),
                time_zone: Some("UTC".to_string()),
                items,
            });
        self
    }
}

impl Calendar for FakeCalendar {
    fn events(&self, calendar: &str, window: &Window) -> Result<Listing, ApiError> {
        self.windows_asked.borrow_mut().push(*window);
        self.listings
            .borrow_mut()
            .get_mut(calendar)
            .and_then(VecDeque::pop_front)
            .ok_or_else(|| {
                ServiceError::Http {
                    service: SERVICE,
                    status: 404,
                    body: format!("calendar {calendar} not found"),
                }
                .into()
            })
    }
}
