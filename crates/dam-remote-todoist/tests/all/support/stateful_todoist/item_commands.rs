use serde_json::Value;

use super::Held;

impl Held {
    pub(super) fn update(&mut self, args: &Value) {
        let updated = self
            .items
            .iter_mut()
            .find(|i| i["id"] == args["id"])
            .unwrap();
        for (field, value) in args.as_object().unwrap() {
            if field != "id" {
                updated[field] = value.clone();
            }
        }
    }

    pub(super) fn close(&mut self, id: &str) {
        self.check(id, true);
    }

    pub(super) fn reopen(&mut self, id: &str) {
        self.check(id, false);
    }

    fn check(&mut self, id: &str, checked: bool) {
        let item = self.items.iter_mut().find(|i| i["id"] == id).unwrap();
        item["checked"] = checked.into();
    }

    pub(super) fn move_item(&mut self, args: &Value) {
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

    pub(super) fn delete_with_everything_under(&mut self, id: &str) {
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
