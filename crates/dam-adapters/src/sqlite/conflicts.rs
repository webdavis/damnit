use dam_application::{
    Conflict, ConflictRepository, Notice, NoticeRepository, RemoteName, StoreError,
};
use dam_domain::{Object, Oid};
use rusqlite::params;

use super::SqliteStore;
use super::codec::{notice_from_json, notice_to_json, object_from_json, object_to_json};
use super::objects::sql;

impl SqliteStore {
    pub(super) fn record_conflict(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        theirs: &Object,
    ) -> Result<(), StoreError> {
        let ours = self
            .get_object(oid)?
            .ok_or_else(|| StoreError("conflict on a missing object".into()))?;
        self.conn
            .execute(
                "INSERT INTO conflicts (oid, remote, ours_json, theirs_json) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(oid) DO UPDATE SET remote = excluded.remote, ours_json = excluded.ours_json, theirs_json = excluded.theirs_json",
                params![oid.as_str(), remote.0, object_to_json(&ours)?, object_to_json(theirs)?],
            )
            .map(|_| ())
            .map_err(sql)
    }

    pub(super) fn all_conflicts(&self) -> Result<Vec<Conflict>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT oid, remote, ours_json, theirs_json FROM conflicts ORDER BY oid")
            .map_err(sql)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(sql)?;
        rows.map(|r| {
            let (oid, remote, ours, theirs) = r.map_err(sql)?;
            Ok(Conflict {
                remote: RemoteName(remote),
                oid: Oid::parse(&oid).map_err(|e| StoreError(e.to_string()))?,
                ours: object_from_json(&ours)?,
                theirs: object_from_json(&theirs)?,
            })
        })
        .collect()
    }

    pub(super) fn drop_conflict(&self, oid: &Oid) -> Result<(), StoreError> {
        self.conn
            .execute(
                "DELETE FROM conflicts WHERE oid = ?1",
                params![oid.as_str()],
            )
            .map(|_| ())
            .map_err(sql)
    }

    pub(super) fn push_notice(&self, notice: &Notice) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO notices (json) VALUES (?1)",
                params![notice_to_json(notice)?],
            )
            .map(|_| ())
            .map_err(sql)
    }

    pub(super) fn all_notices(&self) -> Result<Vec<Notice>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM notices ORDER BY id")
            .map_err(sql)?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(sql)?;
        rows.map(|r| r.map_err(sql).and_then(|j| notice_from_json(&j)))
            .collect()
    }

    pub(super) fn drop_notices(&self) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM notices", [])
            .map(|_| ())
            .map_err(sql)
    }

    pub(super) fn retries_of(&self, remote: &RemoteName) -> Result<Vec<Oid>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT oid FROM push_retries WHERE remote = ?1 ORDER BY oid")
            .map_err(sql)?;
        let rows = stmt
            .query_map(params![remote.0], |r| r.get::<_, String>(0))
            .map_err(sql)?;
        rows.map(|r| {
            r.map_err(sql)
                .and_then(|o| Oid::parse(&o).map_err(|e| StoreError(e.to_string())))
        })
        .collect()
    }

    pub(super) fn set_retries(&self, remote: &RemoteName, oids: &[Oid]) -> Result<(), StoreError> {
        self.in_savepoint("dam_retries", || {
            self.conn
                .execute(
                    "DELETE FROM push_retries WHERE remote = ?1",
                    params![remote.0],
                )
                .map_err(sql)?;
            for oid in oids {
                self.conn
                    .execute(
                        "INSERT INTO push_retries (remote, oid) VALUES (?1, ?2)",
                        params![remote.0, oid.as_str()],
                    )
                    .map_err(sql)?;
            }
            Ok(())
        })
    }
}

impl ConflictRepository for SqliteStore {
    fn mark_conflict(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        theirs: &Object,
    ) -> Result<(), StoreError> {
        self.record_conflict(remote, oid, theirs)
    }
    fn conflicts(&self) -> Result<Vec<Conflict>, StoreError> {
        self.all_conflicts()
    }
    fn clear_conflict(&self, oid: &Oid) -> Result<(), StoreError> {
        self.drop_conflict(oid)
    }
}

impl NoticeRepository for SqliteStore {
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError> {
        self.push_notice(notice)
    }
    fn notices(&self) -> Result<Vec<Notice>, StoreError> {
        self.all_notices()
    }
    fn clear_notices(&self) -> Result<(), StoreError> {
        self.drop_notices()
    }
}

#[cfg(test)]
mod tests {
    use crate::SqliteStore;
    use dam_application::{
        CommitRepository, ConflictRepository, Notice, NoticeRepository, ObjectRepository,
        RemoteName,
    };
    use dam_domain::{Object, Oid, Task};

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    fn remote() -> RemoteName {
        RemoteName("todoist".into())
    }

    #[test]
    fn conflicts_carry_ours_and_theirs_and_clear() {
        let s = SqliteStore::in_memory().unwrap();
        s.put(&Object::Task(Task::new(oid(1), "mine"))).unwrap();
        s.mark_conflict(
            &remote(),
            &oid(1),
            &Object::Task(Task::new(oid(1), "theirs")),
        )
        .unwrap();
        let c = &s.conflicts().unwrap()[0];
        assert_eq!(c.ours.base().subject, "mine");
        assert_eq!(c.theirs.base().subject, "theirs");
        s.clear_conflict(&oid(1)).unwrap();
        assert!(s.conflicts().unwrap().is_empty());
    }

    #[test]
    fn notices_keep_insertion_order_and_clear() {
        let s = SqliteStore::in_memory().unwrap();
        s.add_notice(&Notice::EventCancelled {
            oid: oid(1),
            subject: "a".into(),
            attached: 1,
        })
        .unwrap();
        s.add_notice(&Notice::PushFailed {
            remote: remote(),
            oid: oid(2),
            why: "w".into(),
        })
        .unwrap();
        let n = s.notices().unwrap();
        assert!(matches!(n[0], Notice::EventCancelled { .. }));
        assert!(matches!(n[1], Notice::PushFailed { .. }));
        s.clear_notices().unwrap();
        assert!(s.notices().unwrap().is_empty());
    }

    #[test]
    fn push_retries_replace_the_set() {
        let s = SqliteStore::in_memory().unwrap();
        s.set_push_retries(&remote(), &[oid(1), oid(2)]).unwrap();
        assert_eq!(s.push_retries(&remote()).unwrap(), vec![oid(1), oid(2)]);
        s.set_push_retries(&remote(), &[]).unwrap();
        assert!(s.push_retries(&remote()).unwrap().is_empty());
    }
}
