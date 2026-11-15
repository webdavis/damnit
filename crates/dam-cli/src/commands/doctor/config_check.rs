use super::check::{Check, ConfigFiles, Outcome};
use super::found;
use super::machine::Machine;
use crate::args::CheckName;
use crate::commands::token_warnings::token_warnings;
use crate::error::ConfigReason;
use dam_application::ConfigError;

fn fix(error: &ConfigError) -> &'static str {
    match error {
        ConfigError::NotLua(_) => "point --config or DAM_CONFIG at a file that ends in .lua",
        ConfigError::Unreadable { .. } => "make the file it names readable",
        _ => "fix the file it names",
    }
}

pub(super) fn check(machine: &Machine) -> Vec<Check> {
    let file = machine.config_file;
    let mut files = ConfigFiles {
        config: found(file.config_path()),
        secrets: found(&file.secrets_path()),
        warnings: Vec::new(),
    };
    let check = match machine.config() {
        Err(error) => Check::of(
            CheckName::Config,
            Outcome::Issue,
            ConfigReason(error).to_string(),
        )
        .advising(fix(error)),
        Ok(config) => {
            files.warnings = token_warnings(&config.tokens).collect();
            let (result, detail) = said(&files);
            Check::of(CheckName::Config, result, detail)
        }
    };
    vec![check.reading(files)]
}

fn said(files: &ConfigFiles) -> (Outcome, String) {
    let read = match (&files.config, &files.secrets) {
        (None, None) => "no config.lua, defaults in use",
        (None, Some(_)) => "no config.lua, defaults in use; secrets.lua is valid",
        (Some(_), None) => "config.lua is valid, no secrets.lua",
        (Some(_), Some(_)) if files.warnings.is_empty() => "both files are valid",
        (Some(_), Some(_)) => "both files read",
    };
    match files.warnings.len() {
        0 => (Outcome::Ok, read.to_string()),
        1 => (Outcome::Warning, format!("{read}, with 1 token warning")),
        n => (Outcome::Warning, format!("{read}, with {n} token warnings")),
    }
}
