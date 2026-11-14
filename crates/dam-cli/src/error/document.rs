use super::{CliError, misuse};

impl CliError {
    pub(crate) fn document(&self) -> serde_json::Value {
        let mut error = serde_json::json!({
            "kind": self.kind(),
            "rule": self.rule(),
            "message": self.to_string(),
            "oids": self.oids_in_the_order_the_message_names_them(),
        });
        if let Some(usage) = misuse::usage_part(self) {
            error["usage"] = usage;
        }
        serde_json::json!({ "error": error })
    }
}
