use std::time::Instant;

use dam_application::{
    EditRequest, FunctionFailure, FunctionFault, Histories, HookEvent, Hooks, Pull,
};
use dam_domain::Object;
use mlua::{Function, Value};

use super::functions::{LuaFunctions, json_value, line_defined};
use super::hook_marker::while_marked;
use super::shape::described;

impl LuaFunctions {
    fn hook(&self, event: HookEvent) -> Result<(LuaFunctions, Function), FunctionFailure> {
        match self.hooks().iter().find(|hook| hook.event == event) {
            Some(hook) => Ok((self.at(&hook.path), hook.read.run.clone())),
            None => Err(self.unkept(&mlua::Error::runtime(format!(
                "no hook {} is kept",
                event.as_str()
            )))),
        }
    }
}

impl Hooks for LuaFunctions {
    fn run(
        &self,
        event: HookEvent,
        task: &Object,
        histories: &dyn Histories,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        let (hook, function) = self.hook(event)?;
        let defined = line_defined(&function);
        hook.limited(|scope, deadline| {
            let returned = while_marked(&hook.lua, event, || {
                hook.call(scope, &function, task, None, histories)
            });
            hook.answer(returned, defined, deadline)
        })
    }

    fn pulled(
        &self,
        task: &Object,
        before: Option<&Object>,
        pull: &Pull,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        let event = HookEvent::TaskPulled;
        let (hook, function) = self.hook(event)?;
        let defined = line_defined(&function);
        hook.limited(|_, deadline| {
            let returned = while_marked(&hook.lua, event, || {
                let pulled = hook.lua.create_table()?;
                pulled.set("remote", pull.remote.0.as_str())?;
                pulled.set("change", pull.change.as_str())?;
                let before = before.map(|b| hook.object_value(b, None)).transpose()?;
                function.call((hook.object_value(task, None)?, before, pulled))
            });
            hook.answer(returned, defined, deadline)
        })
    }
}

impl LuaFunctions {
    fn answer(
        &self,
        returned: mlua::Result<Value>,
        defined: Option<u32>,
        deadline: Instant,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        if Instant::now() >= deadline {
            return Err(self.failed(defined, FunctionFault::RanOver));
        }
        match returned {
            Ok(Value::Nil) => Ok(None),
            Ok(table @ Value::Table(_)) => self.request(&table, defined).map(Some),
            Ok(other) => {
                let returned = described(&other);
                Err(self.failed(defined, FunctionFault::Returned { returned }))
            }
            Err(e) => Err(self.raised(&e, defined)),
        }
    }

    fn request(&self, table: &Value, line: Option<u32>) -> Result<EditRequest, FunctionFailure> {
        let request = |json: String| EditRequest {
            json,
            file: self.path().to_path_buf(),
            line,
        };
        match json_value("the request", table) {
            Ok(document) => Ok(request(document.to_string())),
            Err(why) => Err(request(String::new()).failed(why)),
        }
    }
}
