use dam_application::ProjectSettings;
use dam_domain::{Categories, Category};

use super::super::tests::new_args;
use super::super::{dispatch_in, following};
use super::{global, inside};
use crate::args::{AuthorKind, Command, LsArgs, NewArgs};
use crate::context::Context;
use crate::project_scope::Standing;
use crate::testing::context;

fn area(name: &str, values: &[&str]) -> Categories {
    Categories::new(vec![Category {
        name: name.into(),
        values: values.iter().map(|v| v.to_string()).collect(),
        exclusive: true,
    }])
    .unwrap()
}

fn with_an_area() -> Standing {
    inside(ProjectSettings {
        categories: area("area", &["bug", "feature"]),
        ..ProjectSettings::default()
    })
}

fn labelled(labels: &[&str]) -> NewArgs {
    let mut args = new_args("crash", "");
    args.labels = labels.iter().map(|l| l.to_string()).collect();
    args
}

fn run(ctx: Context, command: Command, standing: Standing) -> Result<(), String> {
    let ctx = following(ctx, &command, &standing).map_err(|e| e.to_string())?;
    dispatch_in(&ctx, command, standing)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn inside_the_project_an_exclusive_category_allows_one_of_its_labels() {
    let both = Command::New(labelled(&["bug", "feature"]));
    assert!(run(context(), both, with_an_area()).is_err());
    let one = Command::New(labelled(&["bug"]));
    assert_eq!(run(context(), one, with_an_area()), Ok(()));
}

#[test]
fn with_global_the_projects_categories_do_not_apply() {
    let both = global(labelled(&["bug", "feature"]));
    assert_eq!(run(context(), both, with_an_area()), Ok(()));
}

#[test]
fn a_query_typed_inside_the_project_may_name_its_category() {
    let ls = Command::Ls(LsArgs {
        query: Some("area:bug".into()),
        by: AuthorKind::All,
        scope: Default::default(),
        place: Default::default(),
    });
    assert_eq!(run(context(), ls, with_an_area()), Ok(()));
}

#[test]
fn a_project_category_clashing_with_a_personal_one_refuses_the_file_quoting_no_label() {
    for personal in [area("area", &["chore"]), area("size", &["feature"])] {
        let mut ctx = context();
        ctx.config.categories = personal;
        let refused = run(ctx, Command::New(labelled(&[])), with_an_area()).unwrap_err();
        assert!(refused.contains("/work/app/.dam.lua"), "{refused}");
        assert!(refused.contains("config.lua"), "{refused}");
        for quoted in ["area", "size", "feature", "chore", "bug"] {
            assert!(!refused.contains(quoted), "{refused}");
        }
    }
}
