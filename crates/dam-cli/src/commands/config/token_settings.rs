use dam_application::{Standing, TokenConfig, TokenTerms};
use dam_domain::Timestamp;

use super::setting::{Setting, Value};
use crate::commands::token::grant_view::GrantView;

pub(super) fn token(token: &TokenConfig) -> (String, Setting) {
    let setting = match &token.standing {
        Standing::Usable(terms) => Setting::part(settings_of(terms)),
        Standing::Disabled(_) => Setting::set(Value::Text("disabled".to_string())),
    };
    (token.name.clone(), setting)
}

fn settings_of(terms: &TokenTerms) -> Vec<(String, Setting)> {
    let grants = terms.grants.iter().map(GrantView::of).collect();
    let via = Value::List(terms.via.iter().map(|c| c.as_str().to_string()).collect());
    let via = match terms.via_written {
        true => Setting::set(via),
        false => Setting::by_default(via),
    };
    let require = match terms.require.is_empty() {
        true => Setting::unset(),
        false => Setting::set(Value::List(terms.require.clone())),
    };
    vec![
        ("grant".to_string(), Setting::set(Value::Grants(grants))),
        instant("expires", terms.expires),
        instant("not_before", terms.not_before),
        ("via".to_string(), via),
        ("require".to_string(), require),
    ]
}

fn instant(name: &str, at: Option<Timestamp>) -> (String, Setting) {
    let setting = match at {
        Some(at) => Setting::set(Value::Text(at.to_string())),
        None => Setting::unset(),
    };
    (name.to_string(), setting)
}
