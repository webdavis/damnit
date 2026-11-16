use dam_application::ProjectSettings;
use dam_domain::{Categories, Category};

use super::super::tests::new_args;
use super::super::{dispatch_in, following};
use super::{global, inside};
use crate::args::{Command, NewArgs};
use crate::context::Context;
use crate::project_scope::Standing;
use crate::testing::{context, list_args};

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
    let ctx = following(ctx, &command, &standing);
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
fn a_category_typed_inside_the_project_may_name_its_group() {
    let list = Command::List(list_args(&["--category", "area=bug"]));
    assert_eq!(run(context(), list, with_an_area()), Ok(()));
}

fn yours(categories: Categories) -> Context {
    let mut ctx = context();
    ctx.config.categories = categories;
    ctx
}

fn list(category: &str, global: bool) -> Command {
    let mut args = list_args(&["--category", category]);
    args.scope.global = global;
    Command::List(args)
}

#[test]
fn inside_the_project_its_category_wins_over_yours_of_the_same_name() {
    let both = Command::New(labelled(&["bug", "feature"]));
    assert!(
        run(
            yours(area("area", &["chore", "task"])),
            both,
            with_an_area()
        )
        .is_err()
    );
    let yours_both = Command::New(labelled(&["chore", "task"]));
    assert_eq!(
        run(
            yours(area("area", &["chore", "task"])),
            yours_both,
            with_an_area()
        ),
        Ok(())
    );
}

#[test]
fn inside_the_project_yours_sharing_a_label_is_set_aside_whole() {
    let yours_both = Command::New(labelled(&["feature", "small"]));
    let size = area("size", &["feature", "small"]);
    assert!(run(yours(size.clone()), yours_both, with_an_area()).is_ok());
    let outside = global(labelled(&["feature", "small"]));
    assert!(run(yours(size), outside, with_an_area()).is_err());
}

#[test]
fn inside_the_project_your_unrelated_category_still_applies() {
    let both = Command::New(labelled(&["small", "big"]));
    assert!(run(yours(area("size", &["small", "big"])), both, with_an_area()).is_err());
}

#[test]
fn with_global_your_category_of_the_same_name_applies() {
    let both = global(labelled(&["chore", "task"]));
    assert!(
        run(
            yours(area("area", &["chore", "task"])),
            both,
            with_an_area()
        )
        .is_err()
    );
}

#[test]
fn a_category_flag_reads_the_projects_group_inside_and_yours_with_global() {
    let mine = || yours(area("area", &["chore"]));
    assert_eq!(run(mine(), list("area=bug", false), with_an_area()), Ok(()));
    assert!(run(mine(), list("area=bug", true), with_an_area()).is_err());
    assert_eq!(
        run(mine(), list("area=chore", true), with_an_area()),
        Ok(())
    );
}
