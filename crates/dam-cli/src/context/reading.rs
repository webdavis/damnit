use dam_adapters::{
    FileSystemTemplates, Interrupt, LuaConfigFile, OsRandom, ProcessLauncher, SystemClock,
};
use dam_application::{
    Access, BackgroundSync, ConfigFile, Invocation, LoadedConfig, RunRecords, SettingsFile, Store,
    TaskTemplates,
};

use super::{
    ChangedByHook, Context, HookWarningsOnStandardError,
    terminal_pair_for_a_human_and_refusals_for_a_machine,
};
use crate::output::Format;
use crate::progress::ProgressOnStandardError;

pub(super) struct Opened {
    pub(super) config_file: Box<dyn ConfigFile>,
    pub(super) settings_file: Box<dyn SettingsFile>,
    pub(super) loaded: LoadedConfig,
    pub(super) store: Box<dyn Store>,
    pub(super) access: Box<dyn Access>,
    pub(super) progress: ProgressOnStandardError,
    pub(super) background: Box<dyn BackgroundSync>,
    pub(super) run_records: Box<dyn RunRecords>,
    pub(super) invocation: Invocation,
    pub(super) templates: Box<dyn TaskTemplates>,
}

impl Context {
    pub(crate) fn reading(
        config_file: LuaConfigFile,
        loaded: LoadedConfig,
        store: Box<dyn Store>,
        progress: ProgressOnStandardError,
        interrupt: &Interrupt,
    ) -> Context {
        let access = Box::new(dam_application::Owner);
        let opened = Opened {
            templates: own_templates(&config_file),
            config_file: Box::new(config_file.clone()),
            settings_file: Box::new(config_file),
            loaded,
            store,
            access,
            progress,
            background: dam_adapters::no_background_sync(),
            run_records: dam_adapters::no_run_records(),
            invocation: Invocation::default(),
        };
        Context::assembled(opened, Format::Json, interrupt)
    }

    pub(super) fn assembled(opened: Opened, format: Format, interrupt: &Interrupt) -> Context {
        let Opened {
            config_file,
            settings_file,
            loaded,
            store,
            access,
            progress,
            background,
            run_records,
            invocation,
            templates,
        } = opened;
        let (prompt, editor) =
            terminal_pair_for_a_human_and_refusals_for_a_machine(format, interrupt);
        Context {
            store,
            config: loaded.config,
            access,
            config_file,
            settings_file,
            clock: Box::new(SystemClock),
            random: Box::new(OsRandom),
            launcher: Box::new(ProcessLauncher::from_env(interrupt.clone())),
            credentials: loaded.credentials,
            filter_functions: loaded.filter_functions,
            hooks: loaded.hooks,
            editor,
            prompt,
            tz: jiff::tz::TimeZone::system(),
            changed_by_hook: ChangedByHook::default(),
            interrupt: interrupt.clone(),
            progress,
            hook_warnings: HookWarningsOnStandardError::for_format(format),
            background,
            run_records,
            invocation,
            templates,
            project_hooks: None,
        }
    }
}

pub(super) fn own_templates(config_file: &LuaConfigFile) -> Box<dyn TaskTemplates> {
    let config = config_file.config_path();
    let folder = config.parent().unwrap_or(config).to_path_buf();
    Box::new(FileSystemTemplates::new(None, folder))
}
