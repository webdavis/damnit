use dam_application::ConfigError;

use super::ConfigText;

mod edits;
mod filters;
mod hooks;
mod project;
mod reads;

fn text(refusal: ConfigError) -> String {
    ConfigText(&refusal).to_string()
}
