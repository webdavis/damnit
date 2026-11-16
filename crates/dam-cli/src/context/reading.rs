use dam_adapters::{Interrupt, LuaConfigFile, OsRandom, ProcessLauncher, SystemClock};
use dam_application::{Access, ConfigFile, LoadedConfig, SettingsFile, Store};

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
            config_file: Box::new(config_file.clone()),
            settings_file: Box::new(config_file),
            loaded,
            store,
            access,
            progress,
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
        }
    }
}
