use std::sync::{Arc, Mutex};

use dam_helper::testing::{self, Loopback, Reply, Seen};
use serde_json::{Value, json};

pub struct Held {
    pub projects: Vec<Value>,
    pub sections: Vec<Value>,
    pub items: Vec<Value>,
    issued: u32,
}

impl Held {
    pub fn new(projects: Vec<Value>, sections: Vec<Value>, items: Vec<Value>) -> Held {
        Held {
            projects,
            sections,
            items,
            issued: 0,
        }
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
        for c in commands {
            let args = &c["args"];
            match c["type"].as_str().unwrap_or_default() {
                "item_add" => {
                    self.issued += 1;
                    let id = format!("n{}", self.issued);
                    issued_ids.insert(c["temp_id"].as_str().unwrap().into(), id.clone().into());
                    self.items.push(json!({"id": id, "content": args["content"], "project_id": args["project_id"], "section_id": args["section_id"], "parent_id": args["parent_id"], "priority": 1, "checked": false, "is_deleted": false}));
                }
                "item_delete" => self.delete_with_everything_under(args["id"].as_str().unwrap()),
                "item_move" => self.move_item(args),
                other => {
                    let refused = json!({"error": format!("this fake does not model {other}")});
                    verdicts.insert(c["uuid"].as_str().unwrap().into(), refused);
                    continue;
                }
            }
            verdicts.insert(c["uuid"].as_str().unwrap().into(), "ok".into());
        }
        json!({"sync_status": verdicts, "temp_id_mapping": issued_ids})
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

pub fn serve(held: Held) -> (Loopback, Arc<Mutex<Held>>) {
    let state = Arc::new(Mutex::new(held));
    let shared = state.clone();
    let server = testing::serve_with(move |seen: &Seen| {
        let mut held = shared.lock().unwrap();
        match seen.form().get("commands") {
            Some(commands) => Reply::json(
                200,
                held.write(&serde_json::from_str::<Vec<Value>>(commands).unwrap()),
            ),
            None => Reply::json(200, held.read()),
        }
    });
    (server, state)
}
