use std::time::Instant;

use dam_application::{EditRequest, FunctionFailure, FunctionFault, Histories, HookEvent, Hooks};
use dam_domain::Object;
use mlua::{Function, Table, Value};

use super::evaluate::described;
use super::functions::{LuaFunctions, json_value, line_defined};
use super::hook_marker::while_marked;

impl LuaFunctions {
    fn hook(&self, event: HookEvent) -> mlua::Result<Function> {
        match self.settings()?.get::<Table>("hook")?.get(event.as_str())? {
            Value::Table(hook) => hook.get("run"),
            Value::Function(run) => Ok(run),
            other => Err(mlua::Error::runtime(format!(
                "hook {} is {}",
                event.as_str(),
                described(&other)
            ))),
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
        let function = self.hook(event).map_err(|e| self.unkept(&e))?;
        let defined = line_defined(&function);
        self.limited(|scope, deadline| {
            let returned = while_marked(&self.lua, event, || {
                self.call(scope, &function, task, histories)
            });
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
        })
    }
}

impl LuaFunctions {
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
