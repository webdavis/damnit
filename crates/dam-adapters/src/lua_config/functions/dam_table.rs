use dam_application::{Histories, History};
use dam_domain::Oid;
use mlua::{Lua, Scope, Table};

use super::history_document::history_document;
use super::lua_value::lua_value;

pub(super) fn dam_table<'scope>(
    lua: &Lua,
    scope: &'scope Scope<'scope, '_>,
    oid: &Oid,
    histories: &'scope dyn Histories,
    owner: &'scope str,
) -> mlua::Result<Table> {
    let (handed, oid) = (oid.to_string(), oid.clone());
    let read = scope.create_function(move |lua, task: Table| {
        if task.get::<String>("oid")? != handed {
            return Err(mlua::Error::runtime(
                "dam.history reads only the task match was handed",
            ));
        }
        match histories.of(&oid) {
            History::Recorded(commits) => lua_value(lua, &history_document(&commits, owner)),
            History::Withheld => Err(mlua::Error::runtime(
                "dam.history is not open to this caller",
            )),
            History::Unread => Err(mlua::Error::runtime("dam.history could not read the log")),
        }
    })?;
    let dam = lua.create_table()?;
    dam.set("history", read)?;
    Ok(dam)
}
