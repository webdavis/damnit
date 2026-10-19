use dam_application::list;

use crate::args::LsArgs;
use crate::commands::remote::refresh_stale_unless_no_pull;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{ObjectList, report};

pub(crate) fn run_ls(ctx: &Context, args: LsArgs) -> Result<Report, CliError> {
    refresh_stale_unless_no_pull(ctx)?;
    let objects = list(
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        &ctx.config,
        args.query.as_deref(),
    )?;
    report(&ObjectList {
        objects: &objects,
        zone: &ctx.tz,
    })
}

#[cfg(test)]
mod tests {
    use crate::args::LsArgs;
    use crate::testing::context;
    use dam_application::FilterConfig;
    use dam_domain::{Categories, Category, Object, Oid, Task, When};
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
            super::run_ls(&ctx, LsArgs { query: None }).unwrap().data["objects"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            super::run_ls(
                &ctx,
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
            &ctx,
            LsArgs {
                query: Some("now".into()),
            },
        )
        .unwrap();
        assert_eq!(saved.human.lines().count(), 1);
        assert!(saved.human.contains("today"));
    }

    #[test]
    fn a_value_of_another_category_is_refused_and_names_both() {
        let mut ctx = context();
        ctx.config.categories = Categories::new(vec![Category {
            name: "effort".into(),
            values: vec!["deep".into()],
            exclusive: true,
        }])
        .unwrap();
        let err = super::run_ls(
            &ctx,
            LsArgs {
                query: Some("effort:home".into()),
            },
        )
        .unwrap_err();
        assert_eq!(err.exit_code(), 4);
        assert_eq!(err.to_string(), "home is not a value of category effort");
    }

    #[test]
    fn an_undeclared_category_key_is_refused_and_names_the_key() {
        let ctx = context();
        let err = super::run_ls(
            &ctx,
            LsArgs {
                query: Some("mood:happy".into()),
            },
        )
        .unwrap_err();
        assert_eq!(err.exit_code(), 4, "a refusal by dam's own rule");
        assert!(err.to_string().contains("mood"));
    }
}
