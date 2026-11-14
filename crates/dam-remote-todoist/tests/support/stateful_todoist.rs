use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use dam_helper::testing::{self, Loopback, Reply, Seen};
use serde_json::{Value, json};

pub mod undocumented;

use undocumented::Undocumented;

#[derive(Clone, Copy)]
pub enum Fault {
    HangUp,
    ServerError,
}

#[derive(Default)]
pub struct Held {
    pub projects: Vec<Value>,
    pub sections: Vec<Value>,
    pub items: Vec<Value>,
    pub uuids_received: Vec<String>,
    issued: u32,
    executed: HashSet<String>,
    issued_id_by_temp_id: HashMap<String, String>,
    fault_after_the_next_write: Option<Fault>,
    refused_once: Option<&'static str>,
    undocumented: Undocumented,
}

impl Held {
    pub fn new(projects: Vec<Value>, sections: Vec<Value>, items: Vec<Value>) -> Held {
        Held {
            projects,
            sections,
            items,
            ..Held::default()
        }
    }

    pub fn applying_the_next_write_then(mut self, fault: Fault) -> Held {
        self.fault_after_the_next_write = Some(fault);
        self
    }

    pub fn behaving(mut self, undocumented: Undocumented) -> Held {
        self.undocumented = undocumented;
        self
    }

    pub fn refusing_once(mut self, command_type: &'static str) -> Held {
        self.refused_once = Some(command_type);
        self
    }

    pub fn item(&self, content: &str) -> Vec<&Value> {
        self.items
            .iter()
            .filter(|i| i["content"] == content)
            .collect()
    }

    fn read(&self) -> Value {
        let open: Vec<&Value> = self.items.iter().filter(|i| i["checked"] != true).collect();
        json!({"sync_token": "t", "full_sync": true, "projects": self.projects, "sections": self.sections, "items": open})
    }

    fn write(&mut self, commands: &[Value]) -> Value {
        let mut verdicts = serde_json::Map::new();
        let mut issued_ids = serde_json::Map::new();
        let mut issued_by_this_write = HashSet::new();
        for c in commands {
            let uuid = c["uuid"].as_str().unwrap().to_string();
            self.uuids_received.push(uuid.clone());
            let kind = c["type"].as_str().unwrap_or_default();
            if self.executed.contains(&uuid) {
                if self.undocumented.replays_the_temp_id_of_a_skipped_command
                    && let Some(temp_id) = c["temp_id"].as_str()
                    && let Some(id) = self.issued_id_by_temp_id.get(temp_id)
                {
                    issued_ids.insert(temp_id.into(), id.clone().into());
                }
                verdicts.insert(uuid, "ok".into());
                continue;
            }
            if self.refused_once == Some(kind) {
                self.refused_once = None;
                verdicts.insert(uuid, json!({"error_code": 20, "error": "refused once"}));
                continue;
            }
            let Some(args) = &self.resolved(&c["args"], &issued_by_this_write) else {
                let refused = json!({"error_code": 15, "error": "Invalid temporary id"});
                verdicts.insert(uuid, refused);
                continue;
            };
            match kind {
                "item_add" => {
                    self.issued += 1;
                    let id = format!("n{}", self.issued);
                    let temp_id = c["temp_id"].as_str().unwrap().to_string();
                    self.issued_id_by_temp_id
                        .insert(temp_id.clone(), id.clone());
                    issued_by_this_write.insert(temp_id.clone());
                    issued_ids.insert(temp_id, id.clone().into());
                    self.items.push(json!({"id": id, "content": args["content"], "project_id": args["project_id"], "section_id": args["section_id"], "parent_id": args["parent_id"], "priority": 1, "checked": false, "is_deleted": false}));
                }
                "item_close" => self.close(args["id"].as_str().unwrap()),
                "item_delete" => self.delete_with_everything_under(args["id"].as_str().unwrap()),
                "item_move" => self.move_item(args),
                other => {
                    let refused = json!({"error": format!("this fake does not model {other}")});
                    verdicts.insert(uuid, refused);
                    continue;
                }
            }
            self.executed.insert(uuid.clone());
            verdicts.insert(uuid, "ok".into());
        }
        json!({"sync_status": verdicts, "temp_id_mapping": issued_ids})
    }

    fn close(&mut self, id: &str) {
        let closed = self.items.iter_mut().find(|i| i["id"] == id).unwrap();
        closed["checked"] = true.into();
    }

    fn move_item(&mut self, args: &Value) {
        let (project, section, parent) = if let Some(parent) = args["parent_id"].as_str() {
            let under = self.items.iter().find(|i| i["id"] == parent).unwrap();
            (
                under["project_id"].clone(),
                under["section_id"].clone(),
                parent.into(),
            )
        } else if let Some(section) = args["section_id"].as_str() {
            let within = self.sections.iter().find(|s| s["id"] == section).unwrap();
            (within["project_id"].clone(), section.into(), Value::Null)
        } else {
            (args["project_id"].clone(), Value::Null, Value::Null)
        };
        let moved = self
            .items
            .iter_mut()
            .find(|i| i["id"] == args["id"])
            .unwrap();
        moved["project_id"] = project;
        moved["section_id"] = section;
        moved["parent_id"] = parent;
    }

    fn delete_with_everything_under(&mut self, id: &str) {
        let under: Vec<String> = self
            .items
            .iter()
            .filter(|i| i["parent_id"] == id)
            .filter_map(|i| i["id"].as_str().map(str::to_string))
            .collect();
        for child in under {
            self.delete_with_everything_under(&child);
        }
        self.items.retain(|i| i["id"] != id);
    }
}

pub fn work_held_upstream() -> Held {
    Held::new(
        vec![
            json!({"id": "work", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}),
        ],
        vec![],
        vec![],
    )
}

pub fn serve(held: Held) -> (Loopback, Arc<Mutex<Held>>) {
    let state = Arc::new(Mutex::new(held));
    let shared = state.clone();
    let server = testing::serve_or_hang_up(move |seen: &Seen| {
        let mut held = shared.lock().unwrap();
        let Some(commands) = seen.form().get("commands").cloned() else {
            return Some(Reply::json(200, held.read()));
        };
        let answer = held.write(&serde_json::from_str::<Vec<Value>>(&commands).unwrap());
        match held.fault_after_the_next_write.take() {
            None => Some(Reply::json(200, answer)),
            Some(Fault::HangUp) => None,
            Some(Fault::ServerError) => Some(Reply::json(502, json!({"error": "Bad gateway"}))),
        }
    });
    (server, state)
}
