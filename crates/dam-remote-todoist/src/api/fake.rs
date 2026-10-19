use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use super::{ApiError, SERVICE, SyncResponse, SyncWrite, TodoistSync};

#[derive(Default)]
pub(crate) struct Recorded {
    pub(crate) executed: Vec<serde_json::Value>,
    pub(crate) uuids_seen: HashSet<String>,
    pub(crate) ids_issued: u32,
    pub(crate) reads: u32,
    pub(crate) writes: u32,
}

impl Recorded {
    pub(crate) fn types(&self) -> Vec<&str> {
        self.executed
            .iter()
            .filter_map(|c| c["type"].as_str())
            .collect()
    }

    pub(crate) fn command(&self, kind: &str) -> serde_json::Value {
        self.executed
            .iter()
            .find(|c| c["type"] == kind)
            .unwrap_or_else(|| panic!("no {kind} in {:?}", self.types()))
            .clone()
    }
}

#[derive(Default)]
pub(crate) struct FakeTodoist {
    tree: serde_json::Value,
    refused_command_type: Option<&'static str>,
    failing_read: RefCell<Option<ApiError>>,
    failing_writes: RefCell<HashMap<u32, ApiError>>,
    pub(crate) recorded: RefCell<Recorded>,
}

pub(crate) fn answered(status: u16, body: &str) -> ApiError {
    ApiError::Http {
        service: SERVICE,
        status,
        body: body.to_string(),
    }
}

pub(crate) fn rate_limited(seconds: u64) -> ApiError {
    ApiError::RateLimited {
        service: SERVICE,
        retry_after: Some(Duration::from_secs(seconds)),
        body: "Rate limit exceeded".to_string(),
    }
}

impl FakeTodoist {
    pub(crate) fn new(tree: serde_json::Value) -> FakeTodoist {
        FakeTodoist {
            tree,
            ..FakeTodoist::default()
        }
    }

    pub(crate) fn refusing(mut self, command_type: &'static str) -> FakeTodoist {
        self.refused_command_type = Some(command_type);
        self
    }

    pub(crate) fn failing_read(self, error: ApiError) -> FakeTodoist {
        self.failing_read.replace(Some(error));
        self
    }

    pub(crate) fn failing_write(self, nth: u32, error: ApiError) -> FakeTodoist {
        self.failing_writes.borrow_mut().insert(nth, error);
        self
    }
}

impl TodoistSync for FakeTodoist {
    fn read_all(&self) -> Result<SyncResponse, ApiError> {
        self.recorded.borrow_mut().reads += 1;
        if let Some(error) = self.failing_read.take() {
            return Err(error);
        }
        serde_json::from_value(self.tree.clone()).map_err(|e| ApiError::decode(SERVICE, e))
    }

    fn write(&self, commands: &[serde_json::Value]) -> Result<SyncWrite, ApiError> {
        let mut recorded = self.recorded.borrow_mut();
        recorded.writes += 1;
        if let Some(error) = self.failing_writes.borrow_mut().remove(&recorded.writes) {
            return Err(error);
        }
        let mut written = SyncWrite::default();
        for c in commands {
            let uuid = c["uuid"].as_str().unwrap_or_default().to_string();
            let kind = c["type"].as_str().unwrap_or_default();
            let verdict = if !recorded.uuids_seen.insert(uuid.clone()) {
                "ok".into()
            } else if self.refused_command_type == Some(kind) {
                serde_json::json!({"error_code": 15, "error": "refused by the test"})
            } else {
                recorded.executed.push(c.clone());
                if let Some(temp) = c["temp_id"].as_str() {
                    recorded.ids_issued += 1;
                    let issued = format!("new{}", recorded.ids_issued);
                    written
                        .issued_id_by_temp_id
                        .insert(temp.to_string(), issued);
                }
                "ok".into()
            };
            written.verdict_by_uuid.insert(uuid, verdict);
        }
        Ok(written)
    }
}
