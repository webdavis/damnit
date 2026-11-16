use dam_application::{Progress, Step, waited_on};
use rusqlite::Connection;

mod parents;
mod steps;

use steps::*;

pub(super) const VERSION: u32 = 16;
const PARENTS_FILLED: u32 = 16;

const UPGRADING: Step<'static> = Step {
    name: "store_upgrade",
    remote: None,
    doing: "upgrading the store",
    did: "upgraded the store",
    unit: "",
    total: None,
};

pub(super) fn upgrade(
    conn: &mut Connection,
    progress: &dyn Progress,
) -> Result<(), rusqlite::Error> {
    let current: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if current == 0 || current >= VERSION {
        return migrate_above_user_version_in_one_transaction(conn);
    }
    waited_on(progress, UPGRADING, || {
        migrate_above_user_version_in_one_transaction(conn)
    })
}

pub(super) fn migrate_above_user_version_in_one_transaction(
    conn: &mut Connection,
) -> Result<(), rusqlite::Error> {
    let current: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if current >= VERSION {
        return Ok(());
    }
    let tx = conn.transaction()?;
    for (version, batch) in BATCHES {
        if current < version {
            tx.execute_batch(batch)?;
        }
    }
    if current < PARENTS_FILLED {
        parents::fill_every_parent(&tx)?;
    }
    tx.pragma_update(None, "user_version", VERSION)?;
    tx.commit()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod author_tests;

#[cfg(test)]
mod pulled_author_tests;

#[cfg(test)]
mod pulled_commit_tests;

#[cfg(test)]
mod upgrade_tests;

#[cfg(test)]
mod pull_failed_tests;

#[cfg(test)]
mod parents_tests;

#[cfg(test)]
mod parents_pending_tests;

#[cfg(test)]
mod parents_unreadable_tests;

#[cfg(test)]
mod upgrade_progress_tests;
