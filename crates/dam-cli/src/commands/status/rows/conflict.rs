use dam_application::Conflict;
use dam_domain::{Object, Oid};
use serde::Serialize;

use crate::view::{Line, display, wire};

#[derive(Serialize)]
pub(in crate::commands::status) struct ConflictRow<'a> {
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    remote: &'a str,
    #[serde(serialize_with = "wire")]
    ours: &'a Object,
    #[serde(serialize_with = "wire")]
    theirs: &'a Object,
}

impl<'a> ConflictRow<'a> {
    pub(in crate::commands::status) fn new(c: &'a Conflict) -> ConflictRow<'a> {
        ConflictRow {
            oid: &c.oid,
            remote: &c.remote.0,
            ours: &c.ours,
            theirs: &c.theirs,
        }
    }

    pub(in crate::commands::status) fn lines(&self) -> [Line; 2] {
        let oid = self.oid.short();
        [
            Line::Row(format!(
                "  {oid}  {}  ours: {:?}  theirs: {:?}",
                self.remote,
                self.ours.base().subject,
                self.theirs.base().subject,
            )),
            Line::Whole(format!("    dam resolve {oid} --ours|--theirs")),
        ]
    }
}
