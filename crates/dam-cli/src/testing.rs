use std::cell::RefCell;

use dam_adapters::Interrupt;
use dam_application::Config;
use dam_application::testing::{FixedClock, FixedRandom, MemoryStore};
use dam_domain::Categories;
use jiff::civil::date;

use crate::context::{Context, terminal_pair_for_a_human_and_refusals_for_a_machine};
use crate::output::Format;

mod config_file;
mod remote_helpers;
mod scripted;
pub(crate) use config_file::FakeConfigFile;
pub(crate) use remote_helpers::{EchoLauncher, FailingLauncher, MissingCredentials};
pub(crate) use scripted::{Save, ScriptedEditor, ScriptedPrompt};

pub(crate) fn context() -> Context {
    Context {
        store: Box::new(MemoryStore::new()),
        config: Config {
            owner: None,
            done_interactive: false,
            remotes: vec![],
            categories: Categories::new(vec![]).unwrap_or_else(|_| unreachable!()),
            filters: vec![],
            tokens: Default::default(),
        },
        access: Box::new(dam_application::Owner),
        config_file: Box::new(FakeConfigFile::default()),
        clock: Box::new(FixedClock(date(2026, 9, 18))),
        random: Box::new(FixedRandom::new(1)),
        launcher: Box::new(EchoLauncher {
            pulled: RefCell::new(None),
        }),
        credentials: Box::new(MissingCredentials),
        editor: Some(Box::new(ScriptedEditor::saving_unchanged())),
        prompt: Box::new(ScriptedPrompt {
            choices: RefCell::new(vec![]),
            texts: RefCell::new(vec![]),
        }),
        tz: jiff::tz::TimeZone::UTC,
        no_pull: false,
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
