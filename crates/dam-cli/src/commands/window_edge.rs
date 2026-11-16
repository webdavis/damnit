use dam_domain::{Timestamp, WindowEdge};

use crate::context::Context;
use crate::error::CliError;

pub(crate) enum Side {
    Opens,
    Closes,
}

pub(crate) fn edge(
    ctx: &Context,
    verb: &str,
    flag: &str,
    text: &str,
    side: Side,
) -> Result<Timestamp, CliError> {
    let now = ctx.clock.now().to_zoned(ctx.tz.clone());
    let read = WindowEdge::read(text);
    let at = read.and_then(|edge| match side {
        Side::Opens => edge.opens(&now),
        Side::Closes => edge.closes(&now),
    });
    at.ok_or_else(|| {
        CliError::Usage(format!(
            "--{flag} cannot be {text:?}; it takes {}; run {verb} --{flag} today",
            WindowEdge::shapes().join(", ")
        ))
    })
}
