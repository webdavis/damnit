use dam_application::{Landed, Sent};
use dam_domain::{ItemHash, Object, Oid, Timestamp};
use serde::Serialize;

use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, display, report};

#[derive(Serialize)]
struct SentView<'a> {
    answer: &'static str,
    #[serde(serialize_with = "display")]
    hash: &'a ItemHash,
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    item: &'static str,
    kind: &'static str,
    subject: &'a str,
    path: &'a str,
    from: &'a str,
    key_id: &'a str,
    channel: &'static str,
    #[serde(serialize_with = "display")]
    created: &'a Timestamp,
}

impl View for SentView<'_> {
    fn human(&self) -> Human {
        let mut cells = vec![self.answer, self.hash.short()];
        if self.oid != self.hash.oid() {
            cells.push(self.oid.short());
        }
        cells.push(self.subject);
        Human::Lines(vec![cells.join("  ")])
    }
}

pub(crate) fn report_landed(
    landed: &Landed,
    written: impl FnOnce(&Object) -> Result<Report, CliError>,
) -> Result<Report, CliError> {
    match landed {
        Landed::Written(object) => written(object),
        Landed::Sent(sent) => report(&view(sent)),
    }
}

fn view(sent: &Sent) -> SentView<'_> {
    let item = &sent.item;
    let base = item.change.object().base();
    SentView {
        answer: sent.answer.as_str(),
        hash: &item.hash,
        oid: item.target(),
        item: item.kind().as_str(),
        kind: item.change.object().kind().as_str(),
        subject: &base.subject,
        path: base.path.as_str(),
        from: &item.author.name,
        key_id: item.author.key_id.as_str(),
        channel: item.author.channel.as_str(),
        created: &item.created,
    }
}
