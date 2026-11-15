mod dam_table;
mod history_document;
mod lua_value;
mod time_limit;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::object_document;
use dam_application::{FunctionFailure, FunctionFault, Histories};
use dam_domain::Object;
use mlua::{Function, Lua, Scope, Table, Value};

use super::evaluate::{ErrorText, line_and_reason, lua_message, short_name_lua_never_truncates};
use super::printed::while_the_top_level_runs;
use dam_table::dam_table;
pub use history_document::history_document;
use lua_value::lua_value;
pub(super) use time_limit::limit_every_thread;

const SETTINGS: &str = "settings";

#[derive(Clone)]
pub struct LuaFunctions {
    lua: Lua,
    path: PathBuf,
    owner: String,
    limit: Duration,
}

impl LuaFunctions {
    pub(super) fn keep(
        lua: Lua,
        path: PathBuf,
        settings: Option<Table>,
        owner: &str,
        limit: Duration,
    ) -> mlua::Result<LuaFunctions> {
        if let Some(settings) = settings {
            lua.set_named_registry_value(SETTINGS, settings)?;
        }
        Ok(LuaFunctions {
            lua,
            path,
            owner: owner.to_string(),
            limit,
        })
    }

    pub(super) fn settings(&self) -> mlua::Result<Table> {
        self.lua.named_registry_value::<Table>(SETTINGS)
    }

    pub(super) fn limited<'env, T>(
        &'env self,
        run: impl for<'s> FnOnce(&'s Scope<'s, 'env>, Instant) -> Result<T, FunctionFailure>,
    ) -> Result<T, FunctionFailure> {
        let deadline = Instant::now() + self.limit;
        time_limit::limit_to(&self.lua, deadline).map_err(|e| self.unkept(&e))?;
        let ran = while_the_top_level_runs(&self.lua, ErrorText::QuotesWhatItsCodeRaised, || {
            let ran = self.lua.scope(|scope| Ok(run(scope, deadline)));
            time_limit::lift(&self.lua);
            ran
        });
        ran.map_err(|e| self.unkept(&e))?
    }

    pub(super) fn call<'s>(
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

    pub(super) fn failed(&self, line: Option<u32>, fault: FunctionFault) -> FunctionFailure {
        FunctionFailure {
            file: self.path.clone(),
            line,
            fault,
        }
    }

    pub(super) fn raised(&self, error: &mlua::Error, defined: Option<u32>) -> FunctionFailure {
        let message = lua_message(error);
        let short_name = short_name_lua_never_truncates(&self.path);
        let (line, message) = match line_and_reason(&short_name, &message) {
            Some((line, reason)) => (line.parse().ok(), reason.to_string()),
            None => (defined, message.clone()),
        };
        self.failed(line, FunctionFault::Raised { message })
    }

    pub(super) fn unkept(&self, error: &mlua::Error) -> FunctionFailure {
        let message = lua_message(error);
        self.failed(None, FunctionFault::Raised { message })
    }
}

pub(super) fn line_defined(function: &Function) -> Option<u32> {
    function.info().line_defined.map(|line| line as u32)
}
