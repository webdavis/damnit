use std::any::TypeId;
use std::ffi::OsString;

use clap::{Arg, ArgAction};
use dam_domain::Priority;
use serde_json::{Map, Value};

use crate::error::CliError;

pub(super) fn tokens(
    command: &clap::Command,
    fields: &Map<String, Value>,
    takes_meta: bool,
) -> Result<Vec<OsString>, CliError> {
    let mut flags = Vec::new();
    for (key, value) in fields {
        if key == "meta" && takes_meta {
            flags.extend(details(value)?);
            continue;
        }
        let arg = command
            .get_arguments()
            .find(|arg| arg.get_id() == key.as_str() && !arg.is_hide_set())
            .ok_or_else(|| CliError::Usage(format!("--request: {key} is not a field it takes")))?;
        if arg.is_positional() {
            continue;
        }
        let long = arg.get_long().expect("every flag has a long name");
        match arg.get_action() {
            ArgAction::SetTrue => match value {
                Value::Bool(true) => flags.push(format!("--{long}")),
                Value::Bool(false) => {}
                _ => return Err(wrong(key, "true or false")),
            },
            ArgAction::Append => match value {
                Value::Array(items) => {
                    for item in items {
                        flags.push(format!("--{long}={}", scalar(arg, key, item)?));
                    }
                }
                _ => return Err(wrong(key, "an array")),
            },
            _ => flags.push(format!("--{long}={}", scalar(arg, key, value)?)),
        }
    }
    flags.push("--".into());
    for arg in command.get_arguments().filter(|arg| arg.is_positional()) {
        match fields.get(arg.get_id().as_str()) {
            Some(value) => flags.push(scalar(arg, arg.get_id().as_str(), value)?),
            None if arg.is_required_set() => {
                return Err(CliError::Usage(format!(
                    "--request: {} is required",
                    arg.get_id()
                )));
            }
            None => {}
        }
    }
    Ok(flags.into_iter().map(OsString::from).collect())
}

fn scalar(arg: &Arg, key: &str, value: &Value) -> Result<String, CliError> {
    let takes_a_number = [TypeId::of::<Priority>(), TypeId::of::<u32>()]
        .iter()
        .any(|number| arg.get_value_parser().type_id() == *number);
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Number(number) if takes_a_number && number.is_u64() => Ok(number.to_string()),
        _ if takes_a_number => Err(wrong(key, "a whole number or a string")),
        _ => Err(wrong(key, "a string")),
    }
}

fn details(value: &Value) -> Result<Vec<String>, CliError> {
    let refused = || {
        CliError::Usage(
            "--request: meta is an object of strings, such as {\"session\": \"abc123\"}".into(),
        )
    };
    let Value::Object(details) = value else {
        return Err(refused());
    };
    details
        .iter()
        .map(|(key, value)| match value {
            Value::String(text) if !key.trim().is_empty() && !key.contains('=') => {
                Ok(format!("--meta={key}={text}"))
            }
            _ => Err(refused()),
        })
        .collect()
}

fn wrong(key: &str, expected: &str) -> CliError {
    CliError::Usage(format!("--request: {key} must be {expected}"))
}
