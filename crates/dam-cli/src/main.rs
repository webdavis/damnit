mod args;
mod caller;
mod commands;
mod context;
mod editor;
mod error;
mod first_run;
mod gate;
mod group_help;
mod hook_warnings;
mod interrupted;
mod layout;
mod output;
mod paint;
mod progress;
mod project_scope;
mod prompt;
mod request;
mod started_by_hook;
mod style;
mod view;

use dam_adapters::{Interrupt, LuaConfigFile, ProcessLauncher, SystemClock};
use dam_application::{Clock, ConfigFile};

use crate::args::{Cli, Command, ConfigArgs};
use crate::caller::{authenticated, caller};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Format;
use crate::project_scope::RunFolders;

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
    let format = Format::of(cli.json, cli.toon).quieted(cli.quiet);
    if !interrupts_installed {
        warn_a_human_that_ctrl_c_will_not_cancel_cleanly(format);
    }
    match run(cli, format, &interrupt) {
        Ok(()) => {}
        Err(e) => {
            let e =
                interrupted::cancelled_when_interrupted_whatever_the_work_reported(e, &interrupt);
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

const TOKEN_VARIABLE: &str = "DAM_TOKEN";

fn run(cli: Cli, format: Format, interrupt: &Interrupt) -> Result<(), CliError> {
    let color = paint::wanted(cli.no_color);
    let asked = cli.progress_asked();
    let config_path = cli.config.unwrap_or_else(dam_adapters::default_config_path);
    let config_file = LuaConfigFile::new(config_path.clone());
    let presented_token = std::env::var_os(TOKEN_VARIABLE);
    let meta = cli.meta.into_iter().collect();
    gate::refuse_any_token_to_an_owner_only_verb(presented_token.is_some(), &cli.command)?;
    let store_path = cli.store.unwrap_or_else(dam_adapters::default_store_path);
    let run = RunFolders::of_this_run(&config_path)?;
    if let Command::Test(args) = &cli.command {
        let launcher = ProcessLauncher::from_env(interrupt.clone());
        let progress = context::progress_without_a_context(&config_file, asked, format);
        let machine = commands::test::Machine {
            config_file: &config_file,
            store_path: &store_path,
            launcher: &launcher,
            progress: &progress,
        };
        return commands::test::print(&args.parts, &machine, format, color);
    }
    if let Some(checks) = commands::doctor::asked(&cli.command) {
        let launcher = ProcessLauncher::from_env(interrupt.clone());
        let progress = context::progress_without_a_context(&config_file, asked, format);
        let machine =
            commands::doctor::Machine::here(&config_file, &store_path, &launcher, &progress)
                .within(run.standing());
        return commands::doctor::print(checks, &machine, format, color);
    }
    config_file.ends_in_lua()?;
    if let Some(group) = group_help::bare(&cli.command) {
        return group_help::print(group);
    }
    if let Command::Config(ConfigArgs { command: Some(c) }) = cli.command {
        let listed = commands::config::run(&config_file, c, &run.standing())?;
        return output::print(format, color, &listed);
    }
    if presented_token.is_none() {
        first_run::create_the_owners_config_when_missing(&config_file, &config_path, format);
    }
    let command = match commands::config_files::answer(&config_file, cli.command)? {
        Ok(report) => return output::print(format, color, &report),
        Err(command) => command,
    };
    if commands::catalogue::only_reads(&command) {
        let config = config_file.load()?.config;
        let now = SystemClock.now();
        let access = caller(
            authenticated(&config, presented_token, now, &meta)?,
            now,
            &meta,
        );
        gate::admit(access.as_ref(), &command)?;
        let report = commands::catalogue::run(&config, command)?;
        return output::print(format, color, &report);
    }
    let standing = run.standing();
    let templates = Box::new(run.templates(&standing));
    let ctx = Context::open(
        config_file,
        presented_token,
        &meta,
        format,
        asked,
        interrupt,
        context::invocation_of(&config_path, &store_path),
    )?
    .with_templates(templates);
    let report = match command {
        Command::Init(args) => commands::init::run(&ctx, args, (&run).into())?,
        command => commands::scoped::dispatch_in(&ctx, command, standing)?,
    };
    output::print(format, color, &report)
}

#[cfg(test)]
mod testing;
