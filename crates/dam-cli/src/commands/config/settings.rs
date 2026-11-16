use dam_application::{Config, FilterConfig};
use dam_domain::Category;

use super::hook_settings::hooks;
use super::match_function::MatchFunction;
use super::remote_settings::remote;
use super::setting::{Setting, Value};
use super::token_settings::token;

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
        ("hook", hooks(config)),
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
        flag("progress", config.progress.is_some(), config.progress()),
        (
            "confirm_above".to_string(),
            config
                .confirm_above
                .map_or_else(Setting::unset, |count| Setting::set(Value::Count(count))),
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

fn category(category: &Category, written: bool) -> (String, Setting) {
    let values = Setting::set(Value::List(category.values.clone()));
    let exclusive = flag("exclusive", written, category.exclusive);
    named(
        &category.name,
        vec![("values".to_string(), values), exclusive],
    )
}

fn filter(filter: &FilterConfig) -> (String, Setting) {
    let mut fields: Vec<_> = filter
        .rule
        .query()
        .map(|q| text("query", q))
        .into_iter()
        .collect();
    if filter.rule.runs_a_function() {
        fields.push(("match".to_string(), Setting::Function(MatchFunction)));
    }
    named(&filter.name, fields)
}

pub(super) fn named(name: &str, fields: Vec<(String, Setting)>) -> (String, Setting) {
    (name.to_string(), Setting::part(fields))
}

pub(super) fn text(name: &str, value: &str) -> (String, Setting) {
    (
        name.to_string(),
        Setting::set(Value::Text(value.to_string())),
    )
}

pub(super) fn optional(name: &str, value: Option<&str>) -> (String, Setting) {
    match value {
        Some(value) => text(name, value),
        None => (name.to_string(), Setting::unset()),
    }
}

pub(super) fn flag(name: &str, written: bool, value: bool) -> (String, Setting) {
    let setting = match written {
        true => Setting::set(Value::Flag(value)),
        false => Setting::by_default(Value::Flag(value)),
    };
    (name.to_string(), setting)
}
