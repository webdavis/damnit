use rusqlite::Connection;

mod steps;

use steps::*;

pub(super) const VERSION: u32 = 14;

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
