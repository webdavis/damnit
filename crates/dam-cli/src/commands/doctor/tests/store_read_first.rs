use std::cell::RefCell;
use std::path::Path;

use dam_adapters::SqliteStore;
use dam_application::testing::task;
use dam_application::{
    CommitRepository, HelperError, HelperLauncher, RemoteConfig, RemoteHelper, Secret,
};
use dam_domain::{Authored, Change, CommitId, CommitRecord, Timestamp};

use super::{FAKE_ONLY, FAKE_TOKEN, Files, Helpers, rows_of};
use crate::args::CheckName;
use crate::commands::check_report::CheckReport;
use crate::commands::doctor::{Machine, SETUP, checks};

struct OnLaunch<F: Fn()> {
    helpers: Helpers,
    then: F,
}

impl<F: Fn()> HelperLauncher for OnLaunch<F> {
    fn launch(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        (self.then)();
        self.helpers.launch(remote, credentials)
    }
}

fn commit_task(store: &SqliteStore, n: u8) {
    let record = CommitRecord {
        id: CommitId::generate(&mut |b: &mut [u8]| b.fill(n)),
        message: format!("task {n}"),
        pulled: false,
        at: Timestamp::UNIX_EPOCH,
        changes: vec![Authored::by_owner(Change::Create { after: task(n) })],
    };
    store.commit(&record).unwrap();
}

fn side_files(folder: &Path) -> Vec<String> {
    let names = std::fs::read_dir(folder).unwrap();
    let names = names.map(|e| e.unwrap().file_name().to_string_lossy().into_owned());
    names.filter(|n| n.starts_with("dam.db-")).collect()
}

fn owed_once_a_helper_started(files: &Files, launcher: &dyn HelperLauncher) -> Vec<String> {
    let store = files.store_path();
    let machine = Machine::new(&files.config, &store, launcher);
    assert!(machine.answers().is_some());
    rows_of(&CheckReport::new(
        "doctor",
        SETUP,
        checks(&[CheckName::Pushes], &machine),
    ))
}

#[test]
fn what_is_owed_is_read_before_a_helper_starts_while_another_dam_writes() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    let store = SqliteStore::open(&files.store_path()).unwrap();
    commit_task(&store, 1);
    commit_task(&store, 2);
    drop(store);
    let launcher = OnLaunch {
        helpers: Helpers::default(),
        then: || commit_task(&SqliteStore::open(&files.store_path()).unwrap(), 3),
    };
    let rows = owed_once_a_helper_started(&files, &launcher);
    assert_eq!(rows[1], "    ⚠ fake: 2 changes still owed");
    assert_eq!(side_files(files.dir.path()), Vec::<String>::new());
}

#[test]
fn a_store_another_dam_holds_open_is_let_go_before_a_helper_starts() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    let holder = SqliteStore::open(&files.store_path()).unwrap();
    commit_task(&holder, 1);
    assert_eq!(side_files(files.dir.path()), ["dam.db-shm", "dam.db-wal"]);
    let holder = RefCell::new(Some(holder));
    let left_once_let_go = RefCell::new(None);
    let launcher = OnLaunch {
        helpers: Helpers::default(),
        then: || {
            holder.borrow_mut().take();
            *left_once_let_go.borrow_mut() = Some(side_files(files.dir.path()));
        },
    };
    let rows = owed_once_a_helper_started(&files, &launcher);
    assert_eq!(left_once_let_go.take(), Some(Vec::new()));
    assert_eq!(rows[1], "    ⚠ fake: 1 change still owed");
}
