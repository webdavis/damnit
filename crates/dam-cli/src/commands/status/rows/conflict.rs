use dam_application::Conflict;
use dam_domain::{Object, Oid};
use serde::Serialize;

use crate::view::{Line, display, optional_wire, wire};

#[derive(Serialize)]
pub(in crate::commands::status) struct ConflictRow<'a> {
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    remote: &'a str,
    #[serde(serialize_with = "optional_wire")]
    ours: Option<&'a Object>,
    #[serde(serialize_with = "wire")]
    theirs: &'a Object,
}

impl<'a> ConflictRow<'a> {
    pub(in crate::commands::status) fn new(c: &'a Conflict) -> ConflictRow<'a> {
        ConflictRow {
            oid: &c.oid,
            remote: &c.remote.0,
            ours: c.ours.as_ref(),
            theirs: &c.theirs,
        }
    }

    pub(in crate::commands::status) fn lines(&self) -> [Line; 2] {
        let oid = self.oid.short();
        let (ours, named) = match self.ours {
            Some(o) => (format!("{:?}", o.base().subject), oid.to_string()),
            None => ("removed".to_string(), self.oid.to_string()),
        };
        [
            Line::Row(format!(
                "  {oid}  {}  ours: {ours}  theirs: {:?}",
                self.remote,
                self.theirs.base().subject,
            )),
            Line::Whole(format!("    dam resolve {named} --ours|--theirs")),
        ]
    }
}
