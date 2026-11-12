use dam_application::{Config, Standing, TokenConfig, TokenTerms};
use dam_domain::Timestamp;
use serde::Serialize;

use super::grant_view::GrantView;
use crate::error::CliError;
use crate::layout::table;
use crate::output::Report;
use crate::view::{Human, Line, View, report};

#[derive(Serialize)]
struct TokenList {
    tokens: Vec<TokenRow>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct TokenRow {
    name: String,
    key_id: String,
    standing: &'static str,
    why: Option<String>,
    expires: Option<String>,
    not_before: Option<String>,
    via: Vec<&'static str>,
    require: Vec<String>,
    grants: Vec<GrantView>,
}

impl TokenRow {
    fn of(entry: &TokenConfig, now: Timestamp) -> TokenRow {
        let (standing, why, terms) = match &entry.standing {
            Standing::Disabled(why) => ("disabled", Some(why.clone()), None),
            Standing::Usable(t) if t.expires.is_some_and(|at| now >= at) => {
                ("expired", None, Some(t))
            }
            Standing::Usable(t) if t.not_before.is_some_and(|at| now < at) => {
                ("not_yet_valid", None, Some(t))
            }
            Standing::Usable(t) => ("active", None, Some(t)),
        };
        let stamp = |at: Option<Timestamp>| at.map(|at| at.to_string());
        TokenRow {
            name: entry.name.clone(),
            key_id: entry.value.key_id().to_string(),
            standing,
            why,
            expires: stamp(terms.and_then(|t| t.expires)),
            not_before: stamp(terms.and_then(|t| t.not_before)),
            via: terms.map_or_else(Vec::new, |t| t.via.iter().map(|c| c.as_str()).collect()),
            require: terms.map_or_else(Vec::new, |t: &TokenTerms| t.require.clone()),
            grants: terms.map_or_else(Vec::new, |t| t.grants.iter().map(GrantView::of).collect()),
        }
    }

    fn cells(&self) -> Vec<String> {
        let mut conditions = Vec::new();
        if !self.via.is_empty() {
            conditions.push(format!("via {}", self.via.join(", ")));
        }
        if let Some(at) = &self.expires {
            conditions.push(format!("expires {at}"));
        }
        if let Some(at) = &self.not_before {
            conditions.push(format!("not before {at}"));
        }
        if !self.require.is_empty() {
            conditions.push(format!("requires {}", self.require.join(", ")));
        }
        let grants: Vec<String> = self.grants.iter().map(GrantView::text).collect();
        vec![
            self.name.clone(),
            self.key_id.clone(),
            self.standing.to_string(),
            grants.join(" | "),
            conditions.join("; "),
        ]
    }
}

impl View for TokenList {
    fn human(&self) -> Human {
        let rows: Vec<Vec<String>> = self.tokens.iter().map(TokenRow::cells).collect();
        let listed = match rows.is_empty() {
            true => vec![Line::Whole("no tokens declared".to_string())],
            false => table(&rows).into_iter().map(Line::Row).collect(),
        };
        let reasons = self.tokens.iter().filter_map(|t| {
            let why = t.why.as_ref()?;
            Some(Line::Whole(format!("token {} is disabled: {why}", t.name)))
        });
        let warnings = self
            .warnings
            .iter()
            .map(|w| Line::Whole(format!("warning: {w}")));
        Human::Mixed(listed.into_iter().chain(reasons).chain(warnings).collect())
    }
}

pub(super) fn run(config: &Config, now: Timestamp) -> Result<Report, CliError> {
    report(&TokenList {
        tokens: config
            .tokens
            .entries
            .iter()
            .map(|entry| TokenRow::of(entry, now))
            .collect(),
        warnings: config.tokens.warnings.clone(),
    })
}
