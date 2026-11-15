mod args;
mod caller;
mod cancellation;
mod commands;
mod context;
mod editor;
mod error;
mod first_run;
mod gate;
mod group_help;
mod hook_warnings;
mod last_run;
mod layout;
mod output;
mod paint;
mod prompt;
mod request;
mod started_by_hook;
mod style;
mod view;

use std::ffi::OsString;

use dam_adapters::{Interrupt, LuaConfigFile, OsRandom, ProcessLauncher, SystemClock};
use dam_application::{Clock, ConfigFile};

use crate::args::{Cli, Command, SyncArgs};
use crate::caller::{authenticated, caller};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Format;

fn main() {
    let interrupt = Interrupt::default();
    let interrupts_installed = dam_adapters::catch_interrupts(&interrupt);
    let line: Vec<_> = std::env::args_os().collect();
    let cli = match started_by_hook::refuse_a_dam_a_hook_started(&line)
        .and_then(|()| request::parse(line))
    {
        Ok(cli) => cli,
        Err(refused) => {
            output::print_error(refused.format, &refused.error);
            std::process::exit(refused.error.exit_code());
        }
    };
    let format = Format::of(cli.json, cli.toon);
    if !interrupts_installed {
        cancellation::warn_a_human_that_ctrl_c_will_not_cancel_cleanly(format);
    }
    if let Err(e) = run(cli, format, &interrupt) {
        output::print_error(format, &e);
        std::process::exit(e.exit_code());
    }
}

const TOKEN_VARIABLE: &str = "DAM_TOKEN";

fn run(cli: Cli, format: Format, interrupt: &Interrupt) -> Result<(), CliError> {
    let presented_token = std::env::var_os(TOKEN_VARIABLE);
    gate::refuse_any_token_to_an_owner_only_verb(presented_token.is_some(), &cli.command)?;
    last_run::recording(last_run::asked(&cli.command), format, || {
        admitted(cli, format, interrupt, presented_token).map_err(|e| {
            cancellation::cancelled_when_interrupted_whatever_the_work_reported(e, interrupt)
        })
    })
}

fn admitted(
    cli: Cli,
    format: Format,
    interrupt: &Interrupt,
    presented_token: Option<OsString>,
) -> Result<(), CliError> {
    let color = paint::wanted(cli.no_color);
    let config_path = cli.config.unwrap_or_else(dam_adapters::default_config_path);
    let config_file = LuaConfigFile::new(config_path.clone());
    let meta = cli.meta.into_iter().collect();
    let store_path = cli.store.unwrap_or_else(dam_adapters::default_store_path);
    if let Command::Test(args) = &cli.command {
        let launcher = ProcessLauncher::from_env(interrupt.clone());
        return commands::test::print(&args.parts, &config_file, &launcher, format, color);
    }
    if let Some(asked) = commands::doctor::asked(&cli.command) {
        let launcher = ProcessLauncher::from_env(interrupt.clone());
        let machine = commands::doctor::Machine::new(&config_file, &store_path, &launcher);
        return commands::doctor::print(asked, &machine, format, color);
    }
    config_file.ends_in_lua()?;
    if let Command::Config(args) = cli.command {
        let Some(command) = args.command else {
            return group_help::print("config");
        };
        return output::print(
            format,
            color,
            &commands::config::run(&config_file, command)?,
        );
    }
    if let Command::Sync(SyncArgs { command: None }) = cli.command {
        return group_help::print("sync");
    }
    if presented_token.is_none() {
        first_run::create_the_owners_config_when_missing(&config_file, &config_path, format);
    }
    if let Some(change) = commands::sync_schedule::asked(&cli.command) {
        let in_use = commands::sync_schedule::InUse::of(&config_path, &store_path);
        return commands::sync_schedule::print(change, &config_file, in_use, format, color);
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
    let ctx = Context::open(
        Box::new(config_file),
        &store_path,
        presented_token,
        &meta,
        format,
        interrupt,
    )?;
    if let Command::Sync(_) = cli.command {
        let record_last_run = last_run::asked(&cli.command);
        return commands::scheduled_sync::print(&ctx, format, color, record_last_run);
    }
    let report = commands::dispatch_a_verb_that_opens_the_store(&ctx, cli.command)?;
    output::print(format, color, &report)
}

#[cfg(test)]
mod testing;
