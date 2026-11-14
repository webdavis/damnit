use std::path::PathBuf;

use dam_application::{
    ComputeFailure, CredentialError, CredentialSource, CredentialSpec, RemoteName, Secret,
};
use mlua::{Function, Lua, Table, Value};

use super::evaluate::{described, line_and_reason, lua_message, short_name_lua_never_truncates};

const SECRETS: &str = "secrets";

pub struct ConfiguredCredentials {
    lua: Lua,
    path: PathBuf,
}

impl ConfiguredCredentials {
    pub(super) fn keep(
        lua: Lua,
        path: PathBuf,
        secrets: Option<Table>,
    ) -> mlua::Result<ConfiguredCredentials> {
        if let Some(secrets) = secrets {
            lua.set_named_registry_value(SECRETS, secrets)?;
        }
        Ok(ConfiguredCredentials { lua, path })
    }

    fn compute(&self, remote: &RemoteName, name: &str) -> Result<Secret, CredentialError> {
        let function = self
            .function(&remote.0, name)
            .map_err(|_| CredentialError::Missing(name.to_string()))?;
        let defined = function.info().line_defined.unwrap_or_default().to_string();
        let failed = |line: &str, why: ComputeFailure| CredentialError::ComputeFailed {
            remote: remote.clone(),
            name: name.to_string(),
            at: format!("{}:{line}", self.path.display()),
            why,
        };
        let value: Value = function.call(()).map_err(|e| {
            let message = lua_message(&e);
            let short_name = short_name_lua_never_truncates(&self.path);
            let (line, reason) =
                line_and_reason(&short_name, &message).unwrap_or((&defined, &message));
            failed(
                line,
                ComputeFailure::Raised {
                    message: Some(reason.to_string()),
                },
            )
        })?;
        match value {
            Value::String(text) => match text.to_str() {
                Ok(text) => Ok(Secret::from(&*text)),
                Err(_) => Err(failed(&defined, ComputeFailure::ReturnedNonUtf8)),
            },
            other => Err(failed(
                &defined,
                ComputeFailure::ReturnedNotAString {
                    returned: described(&other),
                },
            )),
        }
    }

    fn function(&self, remote: &str, name: &str) -> mlua::Result<Function> {
        self.lua
            .named_registry_value::<Table>(SECRETS)?
            .get::<Table>("remote")?
            .get::<Table>(remote)?
            .get::<Function>(name)
    }
}

impl CredentialSource for ConfiguredCredentials {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError> {
        match spec {
            CredentialSpec::Literal { value, .. } => Ok(value.clone()),
            CredentialSpec::Computed { remote, name } => self.compute(remote, name),
        }
    }
}
