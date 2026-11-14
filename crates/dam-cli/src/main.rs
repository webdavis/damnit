mod args;
mod caller;
mod commands;
mod context;
mod editor;
mod error;
mod gate;
mod layout;
mod output;
mod paint;
mod prompt;
mod request;
mod view;

use std::path::Path;

use dam_adapters::{Interrupt, LuaConfigFile, OsRandom, SystemClock};
use dam_application::{Clock, ConfigFile};

use crate::args::{Cli, Command};
use crate::caller::{authenticated, caller};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Format;

fn main() {
    let interrupt = Interrupt::default();
    let interrupts_installed = dam_adapters::catch_interrupts(&interrupt);
    let cli = match request::parse(std::env::args_os().collect()) {
        Ok(cli) => cli,
        Err(refused) => {
            output::print_error(refused.format, &refused.error);
            std::process::exit(refused.error.exit_code());
        }
    };
    let format = Format::of(cli.json, cli.toon);
    if !interrupts_installed {
        warn_a_human_that_ctrl_c_will_not_cancel_cleanly(format);
    }
    match run(cli, format, &interrupt) {
        Ok(()) => {}
        Err(e) => {
            let e = cancelled_when_interrupted_whatever_the_work_reported(e, &interrupt);
            output::print_error(format, &e);
            std::process::exit(e.exit_code());
        }
    }
}

fn warn_a_human_that_ctrl_c_will_not_cancel_cleanly(format: Format) {
    if format == Format::Human {
        eprintln!("dam: could not install the interrupt handler; Ctrl-C will not cancel cleanly");
    }
}

fn cancelled_when_interrupted_whatever_the_work_reported(
    e: CliError,
    interrupt: &Interrupt,
) -> CliError {
    if interrupt.requested() {
        CliError::Cancelled
    } else {
        e
    }
}

const TOKEN_VARIABLE: &str = "DAM_TOKEN";

fn run(cli: Cli, format: Format, interrupt: &Interrupt) -> Result<(), CliError> {
    let color = paint::wanted(cli.no_color);
    if cli.no_pull && !commands::pulls_a_stale_remote(&cli.command) {
        return Err(CliError::Usage(
            "--no-pull applies to ls, show, agenda and reminders, the reads that pull a stale remote".into(),
        ));
    }
    let config_path = cli.config.unwrap_or_else(dam_adapters::default_config_path);
    if config_path.extension().is_none_or(|e| e != "lua") {
        return Err(CliError::ConfigNotLua(config_path));
    }
    let config_file = LuaConfigFile::new(config_path.clone());
    let presented_token = std::env::var_os(TOKEN_VARIABLE);
    let meta = cli.meta.into_iter().collect();
    gate::refuse_any_token_to_an_owner_only_verb(presented_token.is_some(), &cli.command)?;
    if presented_token.is_none() {
        create_the_owners_config_when_missing(&config_file, &config_path, format);
    }
    if let Command::Config(_) = cli.command {
        return output::print(format, color, &commands::config::check(&config_file)?);
    }
    if let Command::Token(args) = cli.command {
        return output::print(
            format,
            color,
            &commands::token::run(&config_file, &OsRandom, SystemClock.now(), args)?,
        );
    }
    if let Command::Category(_) | Command::Filter(_) = cli.command {
        let config = config_file.load()?.config;
        let now = SystemClock.now();
        let access = caller(
            authenticated(&config, presented_token, now, &meta)?,
            now,
            &meta,
        );
        gate::admit(access.as_ref(), &cli.command)?;
        let report = commands::catalogue::run(&config, cli.command)?;
        return output::print(format, color, &report);
    }
    let store_path = cli.store.unwrap_or_else(dam_adapters::default_store_path);
    let ctx = Context::open(
        Box::new(config_file),
        &store_path,
        presented_token,
        &meta,
        format,
        cli.no_pull,
        interrupt,
    )?;
    let report = commands::dispatch_a_verb_that_opens_the_store(&ctx, cli.command)?;
    output::print(format, color, &report)
}

fn create_the_owners_config_when_missing(file: &LuaConfigFile, path: &Path, format: Format) {
    let said = match file.create_if_missing() {
        Ok(false) => return,
        Ok(true) => format!(
            "dam: created {}, with every setting explained",
            path.display()
        ),
        Err(e) => format!("dam: could not create {}: {e}", path.display()),
    };
    if format == Format::Human {
        eprintln!("{said}");
    }
}

#[cfg(test)]
mod testing;
