use dam_application::{Expected, HookConfig, HookEvent, Misread, OnFailure};
use mlua::Value;

use super::strict::StrictMap;

pub(crate) const HOOK: [&str; 3] = event_names();
pub(crate) const HOOK_TABLE: [&str; 2] = ["run", "on_failure"];

const fn event_names() -> [&'static str; 3] {
    let [new, changed, done] = HookEvent::ALL;
    [new.as_str(), changed.as_str(), done.as_str()]
}

pub(super) fn read_hooks(root: &StrictMap) -> Result<Vec<HookConfig>, Misread> {
    let Some(hooks) = root.map("hook")? else {
        return Ok(Vec::new());
    };
    hooks.only(&HOOK)?;
    HookEvent::ALL
        .into_iter()
        .filter_map(|event| {
            let value = hooks.get(event.as_str())?;
            Some(read_hook(&hooks, event, value))
        })
        .collect()
}

fn read_hook(hooks: &StrictMap, event: HookEvent, value: &Value) -> Result<HookConfig, Misread> {
    let on_failure = match value {
        Value::Function(_) => None,
        Value::Table(_) => {
            let hook = hooks.child(event.as_str(), value)?;
            hook.only(&HOOK_TABLE)?;
            match hook.get("run") {
                Some(Value::Function(_)) => {}
                Some(_) => return Err(hook.wrong("run", Expected::Function)),
                None => return Err(hook.missing("run")),
            }
            read_on_failure(&hook)?
        }
        _ => return Err(hooks.wrong(event.as_str(), Expected::FunctionOrTable)),
    };
    Ok(HookConfig { event, on_failure })
}

fn read_on_failure(hook: &StrictMap) -> Result<Option<OnFailure>, Misread> {
    let Some(written) = hook.string("on_failure")? else {
        return Ok(None);
    };
    OnFailure::ALL
        .into_iter()
        .find(|choice| choice.as_str() == written)
        .map(Some)
        .ok_or_else(|| hook.wrong("on_failure", Expected::WarnOrRefuse))
}
