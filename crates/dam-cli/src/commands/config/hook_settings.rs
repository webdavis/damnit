use dam_application::{Config, HookConfig, HookEvent};

use super::match_function::MatchFunction;
use super::setting::{Setting, Value};

pub(super) fn hooks(config: &Config) -> Vec<(String, Setting)> {
    let pulled = config.task_pulled.map(|_| {
        named(
            HookEvent::TaskPulled.as_str(),
            vec![("run".to_string(), Setting::Function(MatchFunction))],
        )
    });
    config.hooks.iter().map(hook).chain(pulled).collect()
}

fn hook(hook: &HookConfig) -> (String, Setting) {
    let on_failure = Value::Text(hook.on_failure().as_str().to_string());
    let on_failure = match hook.on_failure {
        Some(_) => Setting::set(on_failure),
        None => Setting::by_default(on_failure),
    };
    named(
        hook.event.as_str(),
        vec![
            ("run".to_string(), Setting::Function(MatchFunction)),
            ("on_failure".to_string(), on_failure),
        ],
    )
}

fn named(name: &str, fields: Vec<(String, Setting)>) -> (String, Setting) {
    (name.to_string(), Setting::part(fields))
}
