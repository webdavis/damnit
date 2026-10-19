use dam_application::Restored;
use dam_domain::Oid;
use serde::Serialize;

use crate::args::RestoreArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct RestoreView<'a> {
    #[serde(skip)]
    results: &'a [Restored],
    restored: Vec<String>,
    unchanged: Vec<String>,
}

impl View for RestoreView<'_> {
    fn human(&self) -> Human {
        Human::Lines(self.results.iter().map(line).collect())
    }
}

pub(crate) fn run(ctx: &Context, args: RestoreArgs) -> Result<Report, CliError> {
    let oids = args
        .oids
        .iter()
        .map(|text| ctx.dam().resolve_object(text))
        .collect::<Result<Vec<Oid>, _>>()?;
    let restored = ctx.dam().restore(&oids)?;
    let ids = |changed: bool| {
        restored
            .iter()
            .filter(|r| r.changed == changed)
            .map(|r| r.oid.to_string())
            .collect::<Vec<String>>()
    };
    report(&RestoreView {
        results: &restored,
        restored: ids(true),
        unchanged: ids(false),
    })
}

fn line(r: &Restored) -> String {
    match r.changed {
        true => format!("restored  {}", r.oid.short()),
        false => format!("{} already matches the last commit", r.oid.short()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::context;
    use dam_domain::{Object, Task};

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    fn committed_task(ctx: &Context, b: u8) -> Oid {
        let id = oid(b);
        ctx.store
            .put(&Object::Task(Task::new(id.clone(), "milk")))
            .unwrap();
        ctx.dam().add(std::slice::from_ref(&id)).unwrap();
        ctx.dam().commit("m").unwrap();
        id
    }

    fn args(oids: &[&Oid]) -> RestoreArgs {
        RestoreArgs {
            oids: oids.iter().map(|o| o.as_str().to_string()).collect(),
        }
    }

    #[test]
    fn the_json_document_lists_the_restored_oids() {
        let ctx = context();
        let id = committed_task(&ctx, 1);
        let mut edited = ctx.store.get(&id).unwrap().unwrap();
        edited.base_mut().subject = "oat milk".into();
        ctx.store.put(&edited).unwrap();
        let report = run(&ctx, args(&[&id])).unwrap();
        assert_eq!(report.data["restored"], serde_json::json!([id.to_string()]));
        assert_eq!(report.data["unchanged"], serde_json::json!([]));
        assert_eq!(ctx.store.get(&id).unwrap().unwrap().base().subject, "milk");
    }

    #[test]
    fn an_unchanged_object_is_reported_as_one_and_not_as_restored() {
        let ctx = context();
        let id = committed_task(&ctx, 1);
        let report = run(&ctx, args(&[&id])).unwrap();
        assert_eq!(report.data["restored"], serde_json::json!([]));
        assert_eq!(
            report.data["unchanged"],
            serde_json::json!([id.to_string()])
        );
        assert!(report.human.contains("already matches"), "{}", report.human);
    }

    #[test]
    fn a_repeated_oid_lands_in_one_bucket_only() {
        let ctx = context();
        let id = committed_task(&ctx, 1);
        let mut edited = ctx.store.get(&id).unwrap().unwrap();
        edited.base_mut().subject = "oat milk".into();
        ctx.store.put(&edited).unwrap();
        let report = run(&ctx, args(&[&id, &id])).unwrap();
        assert_eq!(report.data["restored"], serde_json::json!([id.to_string()]));
        assert_eq!(report.data["unchanged"], serde_json::json!([]));
    }

    #[test]
    fn a_removed_object_takes_its_full_oid_and_the_prefix_refusal_says_so() {
        let ctx = context();
        let id = committed_task(&ctx, 1);
        ctx.store.delete(&id).unwrap();
        let err = run(
            &ctx,
            RestoreArgs {
                oids: vec![id.short().to_string()],
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("working layer"), "{err}");
        assert!(err.to_string().contains("full oid"), "{err}");
        run(&ctx, args(&[&id])).unwrap();
        assert!(ctx.store.get(&id).unwrap().is_some());
    }

    #[test]
    fn an_object_with_no_commit_behind_it_is_refused_by_dams_own_rule() {
        let ctx = context();
        let id = oid(7);
        ctx.store
            .put(&Object::Task(Task::new(id.clone(), "fresh")))
            .unwrap();
        let err = run(&ctx, args(&[&id])).unwrap_err();
        assert_eq!(err.exit_code(), 4);
        assert!(err.to_string().contains(id.short()), "{err}");
    }
}
