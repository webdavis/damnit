use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use dam_adapters::{OpenError, ReadOnly, SqliteStore};
use dam_application::{
    AcceptedKindsRepository, AttemptRepository, CommitRepository, Notice, NoticeRepository,
    RemoteConfig, RemoteTrackingRepository, StoreError, UseCaseError,
};
use dam_domain::Timestamp;

use crate::error::UseCaseText;

pub(super) enum Store {
    NoneYet,
    LeftOver(Vec<PathBuf>),
    Read(Reading),
}

pub(super) struct Reading {
    pub(super) version: u32,
    pub(super) owed: Vec<Result<Owed, String>>,
    pub(super) synced: Vec<Result<Synced, String>>,
}

pub(super) struct Owed {
    pub(super) changes: usize,
    pub(super) unanswered: usize,
}

pub(super) struct Synced {
    pub(super) last_pull: Option<Timestamp>,
    pub(super) failed: Option<String>,
}

impl Synced {
    pub(super) const NEVER: Synced = Synced {
        last_pull: None,
        failed: None,
    };
}

pub(super) fn read(path: &Path, remotes: &[RemoteConfig]) -> Result<Store, OpenError> {
    let store = match SqliteStore::open_read_only(path)? {
        ReadOnly::NoStore => return Ok(Store::NoneYet),
        ReadOnly::LeftOver(files) => return Ok(Store::LeftOver(files)),
        ReadOnly::Opened(store) => store,
    };
    let version = store.schema_version()?;
    let remotes = match version == SqliteStore::SCHEMA_VERSION {
        true => remotes,
        false => &[],
    };
    Ok(Store::Read(Reading {
        version,
        owed: each(&store, remotes, owed),
        synced: each(&store, remotes, synced),
    }))
}

fn each<T>(
    store: &SqliteStore,
    remotes: &[RemoteConfig],
    read: fn(&SqliteStore, &RemoteConfig) -> Result<T, StoreError>,
) -> Vec<Result<T, String>> {
    let read_one = |remote| read(store, remote).map_err(said);
    remotes.iter().map(read_one).collect()
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

fn synced(store: &SqliteStore, remote: &RemoteConfig) -> Result<Synced, StoreError> {
    let failed = store
        .notices()?
        .into_iter()
        .find_map(|notice| match notice {
            Notice::SyncFailed { remote: of, why } if of == remote.name => Some(why),
            _ => None,
        });
    Ok(Synced {
        last_pull: store.last_pull(&remote.name)?,
        failed,
    })
}

fn said(error: StoreError) -> String {
    UseCaseText(&UseCaseError::Store(error)).to_string()
}
