use dam_application::list;

use crate::args::LsArgs;
use crate::commands::remote::maybe_pull_stale;
use crate::context::Context;
use crate::error::CliError;
use crate::output::{Report, objects_report};

pub fn run_ls(ctx: &mut Context, args: LsArgs) -> Result<Report, CliError> {
    maybe_pull_stale(ctx)?;
    let objects = list(
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        &ctx.config,
        args.query.as_deref(),
    )?;
    Ok(objects_report(&objects))
}

#[cfg(test)]
mod tests {
    use crate::args::LsArgs;
    use crate::testing::context;
    use dam_application::FilterConfig;
    use dam_domain::{Object, Oid, Task, When};
    use jiff::civil::date;

    #[test]
    fn ls_filters_by_query_and_by_saved_filter() {
        let mut ctx = context();
        let mut due = Task::new(Oid::generate(&mut |x: &mut [u8]| x.fill(1)), "today");
        due.due = Some(When::Day(date(2026, 9, 18)));
        ctx.store.put(&Object::Task(due)).unwrap();
        ctx.store
            .put(&Object::Task(Task::new(
                Oid::generate(&mut |x: &mut [u8]| x.fill(2)),
                "later",
            )))
            .unwrap();
        ctx.config.filters.push(FilterConfig {
            name: "now".into(),
            query: "due:today".into(),
        });
        assert_eq!(
            super::run_ls(&mut ctx, LsArgs { query: None })
                .unwrap()
                .data["objects"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            super::run_ls(
                &mut ctx,
                LsArgs {
                    query: Some("due:today".into())
                }
            )
            .unwrap()
            .data["objects"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let saved = super::run_ls(
            &mut ctx,
            LsArgs {
                query: Some("now".into()),
            },
        )
        .unwrap();
        assert_eq!(saved.human.lines().count(), 1);
        assert!(saved.human.contains("today"));
    }

    #[test]
    fn an_undeclared_category_key_exits_2_and_names_the_key() {
        let mut ctx = context();
        let err = super::run_ls(
            &mut ctx,
            LsArgs {
                query: Some("mood:happy".into()),
            },
        )
        .unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(err.to_string().contains("mood"));
    }
}
