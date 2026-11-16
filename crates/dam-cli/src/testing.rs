use std::cell::RefCell;

use dam_adapters::Interrupt;
use dam_application::Config;
use dam_application::testing::{
    FakeFilterFunctions, FakeHooks, FakeSettingsFile, FixedClock, FixedRandom, MemoryStore,
};
use dam_domain::Categories;
use jiff::civil::date;

use crate::context::{
    Context, HookWarningsOnStandardError, terminal_pair_for_a_human_and_refusals_for_a_machine,
};
use crate::output::Format;
use crate::progress::{ProgressOnStandardError, Shown};

mod config_file;
mod remote_helpers;
mod scheduler;
mod scripted;
pub(crate) use config_file::FakeConfigFile;
pub(crate) use remote_helpers::{EchoLauncher, MissingCredentials};
pub(crate) use scheduler::RecordingScheduler;
pub(crate) use scripted::{Save, ScriptedEditor, ScriptedPrompt};

pub(crate) fn context() -> Context {
    Context {
        store: Box::new(MemoryStore::new()),
        config: Config {
            owner: None,
            done_interactive: None,
            progress: None,
            remotes: vec![],
            categories: Categories::new(vec![]).unwrap_or_else(|_| unreachable!()),
            categories_writing_exclusive: Default::default(),
            filters: vec![],
            hooks: vec![],
            task_pulled: None,
            tokens: Default::default(),
        },
        access: Box::new(dam_application::Owner),
        config_file: Box::new(FakeConfigFile::default()),
        settings_file: Box::new(FakeSettingsFile::default()),
        clock: Box::new(FixedClock(date(2026, 9, 18))),
        random: Box::new(FixedRandom::new(1)),
        launcher: Box::new(EchoLauncher {
            pulled: RefCell::new(None),
        }),
        credentials: Box::new(MissingCredentials),
        filter_functions: Box::new(FakeFilterFunctions::default()),
        hooks: Box::new(FakeHooks::default()),
        editor: Some(Box::new(ScriptedEditor::saving_unchanged())),
        prompt: Box::new(ScriptedPrompt {
            choices: RefCell::new(vec![]),
            texts: RefCell::new(vec![]),
        }),
        tz: jiff::tz::TimeZone::UTC,
        changed_by_hook: Default::default(),
        interrupt: Interrupt::default(),
        progress: ProgressOnStandardError::new(Shown::Hidden, Format::Json),
        hook_warnings: HookWarningsOnStandardError::for_format(Format::Json),
        background: Box::new(RecordingScheduler::default()),
        invocation: dam_application::Invocation {
            dam: "/opt/dam/bin/dam".into(),
            config: "/home/someone/config.lua".into(),
            store: "/home/someone/dam.db".into(),
            search_path: "/usr/bin:/bin".into(),
        },
    }
}

pub(crate) fn machine_context() -> Context {
    let (prompt, editor) =
        terminal_pair_for_a_human_and_refusals_for_a_machine(Format::Json, &Interrupt::default());
    Context {
        prompt,
        editor,
        ..context()
    }
}

pub(crate) fn holding(permissions: &[dam_domain::Permission]) -> Box<dam_application::Holder> {
    Box::new(dam_application::Holder {
        name: "tool".into(),
        grants: dam_domain::Grants {
            holder: dam_domain::Token::generate(&mut |b: &mut [u8]| b.fill(1))
                .key_id()
                .clone(),
            grants: vec![dam_domain::Grant {
                allow: permissions.iter().copied().collect(),
                ..dam_domain::Grant::default()
            }],
            made: Default::default(),
        },
        channel: dam_domain::Channel::Local,
    })
}
