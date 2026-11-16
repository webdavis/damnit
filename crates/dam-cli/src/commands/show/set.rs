use dam_domain::{Item, ItemHash};
use serde::Serialize;

use crate::commands::status::ItemRow;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, Line, View, display, report};

#[derive(Serialize)]
struct ShownSet<'a> {
    #[serde(serialize_with = "display")]
    set: &'a ItemHash,
    items: Vec<ItemRow<'a>>,
    #[serde(skip)]
    owner: &'a str,
}

impl View for ShownSet<'_> {
    fn human(&self) -> Human {
        let short = self.set.short();
        let mut lines: Vec<Line> = self
            .items
            .iter()
            .flat_map(|r| r.lines(self.owner))
            .collect();
        lines.push(Line::Whole(format!(
            "dam add {short} takes the set; dam rm {short} declines it"
        )));
        Human::Mixed(lines)
    }
}

pub(super) fn show_set(ctx: &Context, set: &ItemHash) -> Result<Report, CliError> {
    let members: Vec<Item> = ctx.dam().set_items(set)?;
    let now = ctx.clock.now();
    report(&ShownSet {
        set,
        items: members
            .iter()
            .map(|item| ItemRow::new(item, now, false).told(ctx, item))
            .collect(),
        owner: ctx.config.owner_name(),
    })
}
