mod dam_table;
mod history_document;
mod lua_value;
mod time_limit;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::object_document;
use dam_application::{FilterFunctions, FunctionFailure, FunctionFault, Histories};
use dam_domain::Object;
use mlua::{Function, Lua, Scope, Table, Value};

use super::evaluate::{
    ErrorText, described, line_and_reason, lua_message, short_name_lua_never_truncates,
};
use super::printed::while_the_top_level_runs;
use dam_table::dam_table;
pub use history_document::history_document;
use lua_value::lua_value;
pub(super) use time_limit::limit_every_thread;

const SETTINGS: &str = "settings";

pub struct LuaFilterFunctions {
    lua: Lua,
    path: PathBuf,
    owner: String,
    limit: Duration,
}

impl LuaFilterFunctions {
    pub(super) fn keep(
        lua: Lua,
        path: PathBuf,
        settings: Option<Table>,
        owner: &str,
        limit: Duration,
    ) -> mlua::Result<LuaFilterFunctions> {
        if let Some(settings) = settings {
            lua.set_named_registry_value(SETTINGS, settings)?;
        }
        Ok(LuaFilterFunctions {
            lua,
            path,
            owner: owner.to_string(),
            limit,
        })
    }

    fn function(&self, filter: &str) -> mlua::Result<Function> {
        self.lua
            .named_registry_value::<Table>(SETTINGS)?
            .get::<Table>("filter")?
            .get::<Table>(filter)?
            .get::<Function>("match")
    }

    fn run<'s>(
        &'s self,
        scope: &'s Scope<'s, '_>,
        function: &Function,
        objects: &[Object],
        histories: &'s dyn Histories,
        deadline: Instant,
    ) -> Result<Vec<bool>, FunctionFailure> {
        let defined = function.info().line_defined.map(|line| line as u32);
        let failed = |line, fault| FunctionFailure {
            file: self.path.clone(),
            line,
            fault,
        };
        let mut keeps = Vec::with_capacity(objects.len());
        for object in objects {
            let returned = self.call(scope, function, object, histories);
            if Instant::now() >= deadline {
                return Err(failed(defined, FunctionFault::RanOver));
            }
            keeps.push(match returned {
                Ok(Value::Boolean(kept)) => kept,
                Ok(Value::Nil) => false,
                Ok(other) => {
                    let returned = described(&other);
                    return Err(failed(defined, FunctionFault::Returned { returned }));
                }
                Err(e) => {
                    let message = lua_message(&e);
                    let short_name = short_name_lua_never_truncates(&self.path);
                    let (line, message) = match line_and_reason(&short_name, &message) {
                        Some((line, reason)) => (line.parse().ok(), reason.to_string()),
                        None => (defined, message.clone()),
                    };
                    return Err(failed(line, FunctionFault::Raised { message }));
                }
            });
        }
        Ok(keeps)
    }

    fn call<'s>(
        &'s self,
        scope: &'s Scope<'s, '_>,
        function: &Function,
        object: &Object,
        histories: &'s dyn Histories,
    ) -> mlua::Result<Value> {
        let document =
            serde_json::to_value(object_document(object)).map_err(mlua::Error::external)?;
        let task = lua_value(&self.lua, &document)?;
        let dam = dam_table(&self.lua, scope, object.oid(), histories, &self.owner)?;
        function.call((task, dam))
    }
}

impl FilterFunctions for LuaFilterFunctions {
    fn keeps(
        &self,
        filter: &str,
        objects: &[Object],
        histories: &dyn Histories,
    ) -> Result<Vec<bool>, FunctionFailure> {
        let unkept = |message: String| FunctionFailure {
            file: self.path.clone(),
            line: None,
            fault: FunctionFault::Raised { message },
        };
        let function = self.function(filter).map_err(|e| unkept(lua_message(&e)))?;
        let deadline = Instant::now() + self.limit;
        time_limit::limit_to(&self.lua, deadline).map_err(|e| unkept(lua_message(&e)))?;
        let keeps = while_the_top_level_runs(&self.lua, ErrorText::QuotesWhatItsCodeRaised, || {
            let ran = self
                .lua
                .scope(|scope| Ok(self.run(scope, &function, objects, histories, deadline)));
            time_limit::lift(&self.lua);
            ran
        });
        keeps.map_err(|e| unkept(lua_message(&e)))?
    }
}
