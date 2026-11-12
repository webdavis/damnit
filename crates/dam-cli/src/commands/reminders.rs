use dam_application::{DueReminder, Window};
use dam_domain::Timestamp;
use jiff::tz::TimeZone;
use serde::Serialize;

use crate::args::RemindersArgs;
use crate::commands::parsing::when_flag;
use crate::commands::remote::refresh_stale_unless_no_pull;
use crate::context::Context;
use crate::error::CliError;
use crate::output::{Report, local_time};
use crate::view::{Human, View, display, report};

pub(crate) fn run_reminders(ctx: &Context, args: RemindersArgs) -> Result<Report, CliError> {
    let window = window(ctx, &args)?;
    refresh_stale_unless_no_pull(ctx)?;
    let due = ctx.dam().due_reminders(window, &ctx.tz)?;
    report(&Due {
        reminders: due.iter().map(Row).collect(),
        zone: &ctx.tz,
    })
}

fn window(ctx: &Context, args: &RemindersArgs) -> Result<Window, CliError> {
    let instant = |flag: &str, text: Option<&str>| -> Result<Option<Timestamp>, CliError> {
        text.map(|text| {
            when_flag(ctx, flag, text)?
                .instant(&ctx.tz)
                .ok_or_else(|| CliError::Usage(format!("--{flag}: {text} is outside the calendar")))
        })
        .transpose()
    };
    let start_of_today = ctx.clock.today().to_zoned(ctx.tz.clone()).ok();
    let from = instant("from", args.from.as_deref())?
        .or(start_of_today.map(|z| z.timestamp()))
        .unwrap_or(Timestamp::UNIX_EPOCH);
    let to = instant("to", args.to.as_deref())?.unwrap_or_else(|| ctx.clock.now());
    if to <= from {
        return Err(CliError::Usage("--to must be later than --from".into()));
    }
    Ok(Window { from, to })
}

#[derive(Serialize)]
struct Due<'a> {
    reminders: Vec<Row<'a>>,
    #[serde(skip)]
    zone: &'a TimeZone,
}

struct Row<'a>(&'a DueReminder);

impl Serialize for Row<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Fields<'a> {
            #[serde(serialize_with = "display")]
            oid: &'a dam_domain::Oid,
            kind: &'static str,
            subject: &'a str,
            #[serde(serialize_with = "display")]
            at: &'a Timestamp,
            #[serde(serialize_with = "display")]
            anchor: &'a Timestamp,
            minutes_before: i64,
        }
        let r = self.0;
        Fields {
            oid: &r.oid,
            kind: r.kind.as_str(),
            subject: &r.subject,
            at: &r.at,
            anchor: &r.anchor,
            minutes_before: r.minutes_before,
        }
        .serialize(serializer)
    }
}

impl View for Due<'_> {
    fn human(&self) -> Human {
        let rows = self
            .reminders
            .iter()
            .map(|Row(r)| {
                vec![
                    r.oid.short().to_string(),
                    local_time(r.at, self.zone),
                    format!("{}m before", r.minutes_before),
                    local_time(r.anchor, self.zone),
                    r.kind.as_str().to_string(),
                    r.subject.clone(),
                ]
            })
            .collect();
        Human::list(rows, "no reminders due")
    }
}
