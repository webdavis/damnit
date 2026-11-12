use std::collections::BTreeMap;

use dam_domain::{Author, Details, Object, Oid, Origin};
use serde_json::Value;

use crate::args::AuthorKind;
use crate::commands::status::{DetailsView, visible};
use crate::context::Context;
use crate::error::CliError;

pub(crate) struct Authorship<'a> {
    ctx: &'a Context,
    origins: BTreeMap<Oid, Origin>,
}

impl<'a> Authorship<'a> {
    pub(crate) fn read(ctx: &'a Context) -> Result<Authorship<'a>, CliError> {
        Ok(Authorship {
            ctx,
            origins: ctx.dam().origins()?,
        })
    }

    pub(crate) fn author(&self, oid: &Oid) -> &str {
        match self.origins.get(oid).map(|o| &o.author) {
            Some(Author::Token(token)) => &token.name,
            Some(Author::Owner) | None => self.ctx.config.owner_name(),
        }
    }

    pub(crate) fn details(&self, oid: &Oid) -> Option<&Details> {
        let origin = self.origins.get(oid)?;
        visible(
            self.ctx.access.as_ref(),
            origin.author.token(),
            origin.details.as_ref(),
        )
    }

    pub(crate) fn by(&self, kind: AuthorKind, object: &Object) -> bool {
        let token = self
            .origins
            .get(object.oid())
            .is_some_and(|o| o.author.token().is_some());
        match kind {
            AuthorKind::Mine => !token,
            AuthorKind::Agents => token,
            AuthorKind::All => true,
        }
    }

    pub(crate) fn stamp(&self, oid: &Oid, document: &mut Value) -> Result<(), CliError> {
        let details = serde_json::to_value(self.details(oid).map(DetailsView::of))
            .map_err(|e| CliError::Internal(e.to_string()))?;
        if let Some(fields) = document.as_object_mut() {
            fields.insert("author".into(), self.author(oid).into());
            fields.insert("details".into(), details);
        }
        Ok(())
    }

    pub(crate) fn lines(&self, oid: &Oid) -> Vec<String> {
        let mut lines = vec![format!("author: {}", self.author(oid))];
        let Some(d) = self.details(oid) else {
            return lines;
        };
        lines.extend(d.meta.iter().map(|(key, value)| format!("{key}: {value}")));
        let facts = [
            ("host", d.host.clone()),
            ("dir", d.dir.clone()),
            ("repo", d.repo.clone()),
            ("branch", d.branch.clone()),
            ("at", Some(d.at.to_string())),
            ("channel", Some(d.channel.as_str().to_string())),
        ];
        lines.extend(
            facts
                .into_iter()
                .filter_map(|(key, value)| value.map(|v| format!("{key}: {v}"))),
        );
        lines
    }
}
