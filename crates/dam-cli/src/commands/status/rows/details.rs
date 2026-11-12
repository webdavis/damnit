use std::collections::BTreeMap;

use dam_application::Access;
use dam_domain::{Authored, Details, Timestamp, TokenAuthor};
use serde::Serialize;

use super::ChangeRow;
use crate::context::Context;
use crate::view::display;

#[derive(Serialize)]
pub(crate) struct DetailsView<'a> {
    meta: &'a BTreeMap<String, String>,
    host: Option<&'a str>,
    dir: Option<&'a str>,
    repo: Option<&'a str>,
    branch: Option<&'a str>,
    #[serde(serialize_with = "display")]
    at: &'a Timestamp,
    channel: &'static str,
}

impl<'a> DetailsView<'a> {
    pub(crate) fn of(details: &'a Details) -> DetailsView<'a> {
        DetailsView {
            meta: &details.meta,
            host: details.host.as_deref(),
            dir: details.dir.as_deref(),
            repo: details.repo.as_deref(),
            branch: details.branch.as_deref(),
            at: &details.at,
            channel: details.channel.as_str(),
        }
    }
}

pub(crate) fn visible<'a>(
    reader: &dyn Access,
    sender: Option<&TokenAuthor>,
    details: Option<&'a Details>,
) -> Option<&'a Details> {
    match reader.token() {
        None => details,
        Some(name) => details.filter(|_| sender.is_some_and(|s| s.name == name)),
    }
}

impl<'a> ChangeRow<'a> {
    pub(crate) fn sent(mut self, owner: &'a str, details: Option<&'a Details>) -> ChangeRow<'a> {
        self.author.get_or_insert(owner);
        self.details = details.map(DetailsView::of);
        self
    }

    pub(crate) fn authored(ctx: &'a Context, change: &'a Authored, full: bool) -> ChangeRow<'a> {
        let token = change.author.token();
        let details = visible(ctx.access.as_ref(), token, change.details.as_ref());
        ChangeRow::new(&change.change, token, full).sent(ctx.config.owner_name(), details)
    }
}
