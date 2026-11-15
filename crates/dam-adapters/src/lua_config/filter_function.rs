use std::time::Instant;

use dam_application::{FilterFunctions, FunctionFailure, FunctionFault, Histories};
use dam_domain::Object;
use mlua::{Function, Table, Value};

use super::evaluate::described;
use super::functions::{LuaFunctions, line_defined};

impl LuaFunctions {
    fn filter(&self, filter: &str) -> mlua::Result<Function> {
        self.settings()?
            .get::<Table>("filter")?
            .get::<Table>(filter)?
            .get::<Function>("match")
    }
}

impl FilterFunctions for LuaFunctions {
    fn keeps(
        &self,
        filter: &str,
        objects: &[Object],
        histories: &dyn Histories,
    ) -> Result<Vec<bool>, FunctionFailure> {
        let function = self.filter(filter).map_err(|e| self.unkept(&e))?;
        let defined = line_defined(&function);
        self.limited(|scope, deadline| {
            let mut keeps = Vec::with_capacity(objects.len());
            for object in objects {
                let returned = self.call(scope, &function, object, histories);
                if Instant::now() >= deadline {
                    return Err(self.failed(defined, FunctionFault::RanOver));
                }
                keeps.push(match returned {
                    Ok(Value::Boolean(kept)) => kept,
                    Ok(Value::Nil) => false,
                    Ok(other) => {
                        let returned = described(&other);
                        return Err(self.failed(defined, FunctionFault::Returned { returned }));
                    }
                    Err(e) => return Err(self.raised(&e, defined)),
                });
            }
            Ok(keeps)
        })
    }
}
