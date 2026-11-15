use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use dam_adapters::{OpenError, ReadOnly, SqliteStore};
use dam_application::{
    AcceptedKindsRepository, AttemptRepository, CommitRepository, RemoteConfig, StoreError,
    UseCaseError,
};

use crate::error::UseCaseText;

pub(super) enum Store {
    NoneYet,
    LeftOver(Vec<PathBuf>),
    Read(Reading),
}

pub(super) struct Reading {
    pub(super) version: u32,
    pub(super) owed: Vec<Result<Owed, String>>,
}

pub(super) struct Owed {
    pub(super) changes: usize,
    pub(super) unanswered: usize,
}

pub(super) fn read(path: &Path, remotes: &[RemoteConfig]) -> Result<Store, OpenError> {
    let store = match SqliteStore::open_read_only(path)? {
        ReadOnly::NoStore => return Ok(Store::NoneYet),
        ReadOnly::LeftOver(files) => return Ok(Store::LeftOver(files)),
        ReadOnly::Opened(store) => store,
    };
    let version = store.schema_version()?;
    let owed = match version == SqliteStore::SCHEMA_VERSION {
        true => remotes
            .iter()
            .map(|remote| owed(&store, remote).map_err(said))
            .collect(),
        false => Vec::new(),
    };
    Ok(Store::Read(Reading { version, owed }))
}

fn owed(store: &SqliteStore, remote: &RemoteConfig) -> Result<Owed, StoreError> {
    let accepted = store.accepted_kinds(&remote.name)?;
    let mut oids: BTreeSet<_> = store.push_retries(&remote.name)?.into_iter().collect();
    for record in store.unpushed(&remote.name)? {
        let changes = record.changes.into_iter().map(|authored| authored.change);
        oids.extend(
            changes
                .filter(|c| {
                    accepted
                        .as_ref()
                        .is_none_or(|k| k.contains(&c.object().kind()))
                })
                .map(|c| c.oid().clone()),
        );
    }
    Ok(Owed {
        changes: oids.len(),
        unanswered: store.attempts(&remote.name)?.len(),
    })
}

fn said(error: StoreError) -> String {
    UseCaseText(&UseCaseError::Store(error)).to_string()
}
