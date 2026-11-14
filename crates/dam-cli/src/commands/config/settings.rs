use dam_application::{Config, CredentialSpec, FilterConfig, RemoteConfig, Standing, TokenConfig};
use dam_domain::Category;

use super::setting::{Setting, Value};
use crate::commands::token::grant_view::GrantView;

pub(super) fn settings(config: &Config) -> Setting {
    let owner = match &config.owner {
        Some(owner) => Setting::set(Value::Text(owner.clone())),
        None => Setting::by_default(Value::Text(config.owner_name().to_string())),
    };
    let groups = [
        ("remote", config.remotes.iter().map(remote).collect()),
        (
            "category",
            config
                .categories
                .all()
                .iter()
                .map(|c| category(c, config.categories_writing_exclusive.contains(&c.name)))
                .collect(),
        ),
        ("filter", config.filters.iter().map(filter).collect()),
        ("token", config.tokens.entries.iter().map(token).collect()),
    ];
    let mut entries = vec![
        ("owner".to_string(), owner),
        (
            "done".to_string(),
            Setting::part(vec![flag(
                "interactive",
                config.done_interactive.is_some(),
                config.done_interactive(),
            )]),
        ),
    ];
    entries.extend(
        groups
            .into_iter()
            .filter(|(_, named): &(_, Vec<_>)| !named.is_empty())
            .map(|(group, named)| (group.to_string(), Setting::part(named))),
    );
    Setting::part(entries)
}

fn remote(remote: &RemoteConfig) -> (String, Setting) {
    let deadline = match &remote.deadline {
        Some(deadline) => Setting::set(Value::Text(deadline.text.clone())),
        None => Setting::by_default(Value::Text(RemoteConfig::DEFAULT_DEADLINE.to_string())),
    };
    let mut fields = vec![
        text("url", &remote.url),
        optional(
            "stale",
            remote.stale.as_ref().map(|stale| stale.text.as_str()),
        ),
        ("deadline".to_string(), deadline),
        optional("path", remote.path.as_ref().map(|path| path.as_str())),
    ];
    let credentials: Vec<_> = remote.credentials.iter().map(credential).collect();
    if !credentials.is_empty() {
        fields.push(named("credential", credentials));
    }
    named(&remote.name.0, fields)
}

fn credential(credential: &CredentialSpec) -> (String, Setting) {
    let held = match credential {
        CredentialSpec::Literal { .. } => "set",
        CredentialSpec::Computed { .. } => "function",
    };
    text(credential.name(), held)
}

fn category(category: &Category, written: bool) -> (String, Setting) {
    let values = Setting::set(Value::List(category.values.clone()));
    let exclusive = flag("exclusive", written, category.exclusive);
    named(
        &category.name,
        vec![("values".to_string(), values), exclusive],
    )
}

fn filter(filter: &FilterConfig) -> (String, Setting) {
    named(&filter.name, vec![text("query", &filter.query)])
}

fn token(token: &TokenConfig) -> (String, Setting) {
    let grants = match &token.standing {
        Standing::Usable(terms) => terms
            .grants
            .iter()
            .map(|grant| GrantView::of(grant).text())
            .collect::<Vec<_>>()
            .join(" | "),
        Standing::Disabled(_) => "disabled".to_string(),
    };
    (token.name.clone(), Setting::set(Value::Text(grants)))
}

fn named(name: &str, fields: Vec<(String, Setting)>) -> (String, Setting) {
    (name.to_string(), Setting::part(fields))
}

fn text(name: &str, value: &str) -> (String, Setting) {
    (
        name.to_string(),
        Setting::set(Value::Text(value.to_string())),
    )
}

fn optional(name: &str, value: Option<&str>) -> (String, Setting) {
    match value {
        Some(value) => text(name, value),
        None => (name.to_string(), Setting::unset()),
    }
}

fn flag(name: &str, written: bool, value: bool) -> (String, Setting) {
    let setting = match written {
        true => Setting::set(Value::Flag(value)),
        false => Setting::by_default(Value::Flag(value)),
    };
    (name.to_string(), setting)
}
