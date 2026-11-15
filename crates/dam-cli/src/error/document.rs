use dam_application::{Refusal, UseCaseError};

use super::{CliError, misuse};

impl CliError {
    pub(crate) fn document(&self) -> serde_json::Value {
        let mut error = serde_json::json!({
            "kind": self.kind(),
            "rule": self.rule(),
            "message": self.to_string(),
            "oids": self.oids_in_the_order_the_message_names_them(),
        });
        if let Some(prefix) = self.given_prefix() {
            error["prefix"] = prefix.into();
        }
        if let Some(key) = self.given_key() {
            error["key"] = key.into();
        }
        if let Some(line) = self.crontab_line() {
            error["crontab_line"] = line.into();
        }
        if let Some(usage) = misuse::usage_part(self) {
            error["usage"] = usage;
        }
        if let CliError::UseCase(UseCaseError::Refused(Refusal::HookRefused(refused))) = self {
            error["hook"] = refused.event.as_str().into();
        }
        serde_json::json!({ "error": error })
    }
}
