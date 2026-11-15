use std::time::Instant;

use dam_application::{FunctionFailure, FunctionFault, Histories, HookEvent, Hooks};
use dam_domain::Object;
use mlua::{Function, Table, Value};

use super::evaluate::described;
use super::functions::{LuaFunctions, line_defined};

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
    ) -> Result<(), FunctionFailure> {
        let function = self.hook(event).map_err(|e| self.unkept(&e))?;
        let defined = line_defined(&function);
        self.limited(|scope, deadline| {
            let returned = self.call(scope, &function, task, histories);
            if Instant::now() >= deadline {
                return Err(self.failed(defined, FunctionFault::RanOver));
            }
            match returned {
                Ok(Value::Nil) => Ok(()),
                Ok(other) => {
                    let returned = described(&other);
                    Err(self.failed(defined, FunctionFault::Returned { returned }))
                }
                Err(e) => Err(self.raised(&e, defined)),
            }
        })
    }
}
